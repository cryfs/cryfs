#include "Cli.h"

#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <boost/chrono/duration.hpp>
#include <boost/filesystem/directory.hpp>
#include <boost/filesystem/exception.hpp>
#include <boost/filesystem/operations.hpp>
#include <boost/filesystem/path.hpp>
#include <boost/none.hpp>
#include <boost/optional/optional.hpp>
#include <cmath>
#include <cpp-utils/assert/backtrace.h>
#include <cstdint>

#include "cpp-utils/assert/assert.h"
#include "cpp-utils/crypto/kdf/Scrypt.h"
#include "cpp-utils/io/Console.h"
#include "cpp-utils/logging/logging.h"
#include "cpp-utils/network/HttpClient.h"
#include "cpp-utils/random/RandomGenerator.h"
#include "cpp-utils/tempfile/TempFile.h"
#include "cryfs-cli/CallAfterTimeout.h"
#include "cryfs-cli/program_options/ProgramOptions.h"
#include "cryfs/impl/ErrorCodes.h"
#include "cryfs/impl/config/CryCipher.h"
#include "cryfs/impl/config/CryConfig.h"
#include "cryfs/impl/config/CryConfigFile.h"
#include "gitversion/versionstring.h"
#include "program_options/Parser.h"
#include <cryfs/impl/config/CryConfigLoader.h>
#include <cryfs/impl/config/CryPasswordBasedKeyProvider.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <exception>
#include <fspp/fuse/Fuse.h>
#include <fspp/impl/FilesystemImpl.h>

#include <cryfs/impl/filesystem/CryDir.h>
#include <functional>
#include <gitversion/gitversion.h>

#include "Environment.h"
#include "VersionChecker.h"
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <cpp-utils/thread/debugging.h>
#include <cryfs/impl/CryfsException.h>
#include <cryfs/impl/localstate/BasedirMetadata.h>
#include <cryfs/impl/localstate/LocalStateDir.h>
#include <gitversion/VersionCompare.h>
#include <iostream>
#include <memory>
#include <ostream>
#include <spdlog/sinks/basic_file_sink.h>
#include <spdlog/sinks/stdout_sinks.h>
#include <spdlog/spdlog.h>
#include <stdexcept>
#include <string>
#include <utility>

//TODO Many functions accessing the ProgramOptions object. Factor out into class that stores it as a member.
//TODO Factor out class handling askPassword

using namespace cryfs_cli;
using namespace cryfs;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

using blockstore::ondisk::OnDiskBlockStore2;
using program_options::ProgramOptions;

using cpputils::make_unique_ref;
using cpputils::NoninteractiveConsole;
using cpputils::TempFile;
using cpputils::RandomGenerator;
using cpputils::unique_ref;
using cpputils::SCrypt;
using cpputils::either;
using cpputils::SCryptSettings;
using cpputils::Console;
using cpputils::HttpClient;
using std::cout;
using std::string;
using std::endl;
using std::shared_ptr;
using std::make_shared;
using std::unique_ptr;
using std::make_unique;
using std::function;
using boost::optional;
using boost::none;
using boost::chrono::minutes;
using boost::chrono::milliseconds;
using cpputils::dynamic_pointer_move;
using gitversion::VersionCompare;

//TODO Delete a large file in parallel possible? Takes a long time right now...
//TODO Improve parallelity.
//TODO Replace ASSERTs with other error handling when it is not a programming error but an environment influence (e.g. a block is missing)
//TODO Can we improve performance by setting compiler parameter -maes for scrypt?

namespace cryfs_cli {

    Cli::Cli(RandomGenerator *keyGenerator, const SCryptSettings &scryptSettings, shared_ptr<Console> console):
            _keyGenerator(keyGenerator), _scryptSettings(scryptSettings), _console(), _noninteractive(Environment::isNoninteractive()), _idleUnmounter(none), _device(none) {
        
        if (_noninteractive) {
            _console = make_shared<NoninteractiveConsole>(console);
        } else {
            _console = console;
        }
    }

    void Cli::_showVersion() {
        cout << "CryFS Version " << gitversion::VersionString() << endl;
        if (gitversion::IsDevVersion()) {
            cout << "WARNING! This is a development version based on git commit " << gitversion::GitCommitId() <<
            ". Please do not use in production!" << endl;
        } else if (!gitversion::IsStableVersion()) {
            cout << "WARNING! This is an experimental version. Please backup your data frequently!" << endl;
        }
#ifndef NDEBUG
        cout << "WARNING! This is a debug build. Performance might be slow." << endl;
#endif
        cout << endl;
    }

#ifdef CRYFS_UPDATE_CHECKS
    void Cli::_maybeCheckForUpdates(unique_ref<HttpClient> httpClient) {
        if (Environment::noUpdateCheck()) {
            cout << "Automatic checking for security vulnerabilities and updates is disabled." << endl;
        } else if (Environment::isNoninteractive()) {
            cout << "Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode." << endl;
        } else {
            _checkForUpdates(std::move(httpClient));
        }
    }

    void Cli::_checkForUpdates(unique_ref<HttpClient> httpClient) {
        const VersionChecker versionChecker(httpClient.get());
        optional<string> newestVersion = versionChecker.newestVersion();
        if (newestVersion == none) {
            cout << "Could not check for updates." << endl;
        } else if (VersionCompare::isOlderThan(gitversion::VersionString(), *newestVersion)) {
            cout << "CryFS " << *newestVersion << " is released. Please update." << endl;
        }
        optional<string> securityWarning = versionChecker.securityWarningFor(gitversion::VersionString());
        if (securityWarning != none) {
            cout << *securityWarning << endl;
        }
    }
#endif

    bool Cli::_checkPassword(const string &password) {
        if (password == "") {
            std::cerr << "Empty password not allowed. Please try again." << std::endl;
            return false;
        }
        return true;
    }

    function<string()> Cli::_askPasswordForExistingFilesystem(std::shared_ptr<cpputils::Console> console) {
        return [console] () {
            string password = console->askPassword("Password: ");
            while (!_checkPassword(password)) {
                password = console->askPassword("Password: ");
            }
            return password;
        };
    };

    function<string()> Cli::_askPasswordForNewFilesystem(std::shared_ptr<cpputils::Console> console) {
        //TODO Ask confirmation if using insecure password (<8 characters)
        return [console] () {
            string password;
            bool again = true;
            while(again) {
                password = console->askPassword("Password: ");
                if (!_checkPassword(password)) {
                    again = true;
                    continue;
                }
                if (!_confirmPassword(console.get(), password)) {
                    again = true;
                    continue;
                }
                again = false;
            }
            return password;
        };
    }

    bool Cli::_confirmPassword(cpputils::Console* console, const string &password) {
        const string confirmPassword = console->askPassword("Confirm Password: ");
        if (password != confirmPassword) {
            std::cout << "Passwords don't match" << std::endl;
            return false;
        }
        return true;
    }

    function<string()> Cli::_askPasswordNoninteractive(std::shared_ptr<cpputils::Console> console) {
        //TODO Test
        return [console] () {
            string password = console->askPassword("Password: ");
            if (!_checkPassword(password)) {
                throw CryfsException("Invalid password. Password cannot be empty.", ErrorCode::EmptyPassword);
            }
            return password;
        };
    }

    bf::path Cli::_determineConfigFile(const ProgramOptions &options) {
        auto configFile = options.configFile();
        if (configFile == none) {
            return bf::path(options.baseDir()) / "cryfs.config";
        }
        return *configFile;
    }

    void Cli::_checkConfigIntegrity(const bf::path& basedir, const LocalStateDir& localStateDir, const CryConfigFile& config, bool allowReplacedFilesystem) {
        auto basedirMetadata = BasedirMetadata::load(localStateDir);
        if (!allowReplacedFilesystem && !basedirMetadata.filesystemIdForBasedirIsCorrect(basedir, config.config()->FilesystemId())) {
          if (!_console->askYesNo("The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir. This can be genuine if you replaced the filesystem with a different one. If you didn't do that, it is possible that an attacker did. Do you want to continue loading the file system?", false)) {
            throw CryfsException(
                "The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir.", ErrorCode::FilesystemIdChanged);
          }
        }
        // Update local state (or create it if it didn't exist yet)
        basedirMetadata.updateFilesystemIdForBasedir(basedir, config.config()->FilesystemId());
        basedirMetadata.save();
    }

    CryConfigLoader::ConfigLoadResult Cli::_loadOrCreateConfig(const ProgramOptions &options, const LocalStateDir& localStateDir) {
        auto configFile = _determineConfigFile(options);
        auto config = _loadOrCreateConfigFile(std::move(configFile), localStateDir, options.cipher(), options.blocksizeBytes(), options.allowFilesystemUpgrade(), options.missingBlockIsIntegrityViolation(), options.allowReplacedFilesystem());
        if (config.is_left()) {
            switch(config.left()) {
                case CryConfigFile::LoadError::DecryptionFailed:
                    throw CryfsException("Failed to decrypt the config file. Did you enter the correct password?", ErrorCode::WrongPassword);
                case CryConfigFile::LoadError::ConfigFileNotFound:
                    throw CryfsException("Could not find the cryfs.config file. Are you sure this is a valid CryFS file system?", ErrorCode::InvalidFilesystem);
            }
        }
        _checkConfigIntegrity(options.baseDir(), localStateDir, *config.right().configFile, options.allowReplacedFilesystem());
        return std::move(config.right());
    }

    either<CryConfigFile::LoadError, CryConfigLoader::ConfigLoadResult> Cli::_loadOrCreateConfigFile(bf::path configFilePath, LocalStateDir localStateDir, const optional<string> &cipher, const optional<uint32_t> &blocksizeBytes, bool allowFilesystemUpgrade, const optional<bool> &missingBlockIsIntegrityViolation, bool allowReplacedFilesystem) {
        // TODO Instead of passing in _askPasswordXXX functions to KeyProvider, only pass in console and move logic to the key provider,
        //      for example by having a separate CryPasswordBasedKeyProvider / CryNoninteractivePasswordBasedKeyProvider.
        auto keyProvider = make_unique_ref<CryPasswordBasedKeyProvider>(
          _console,
          _noninteractive ? Cli::_askPasswordNoninteractive(_console) : Cli::_askPasswordForExistingFilesystem(_console),
          _noninteractive ? Cli::_askPasswordNoninteractive(_console) : Cli::_askPasswordForNewFilesystem(_console),
          make_unique_ref<SCrypt>(_scryptSettings)
        );
        return CryConfigLoader(_console, _keyGenerator, std::move(keyProvider), std::move(localStateDir),
                               cipher, blocksizeBytes, missingBlockIsIntegrityViolation).loadOrCreate(std::move(configFilePath), allowFilesystemUpgrade, allowReplacedFilesystem);
    }

    namespace {
        void printConfig(const CryConfig& oldConfig, const CryConfig& updatedConfig) {
            auto printValue = [&] (const char* prefix, const char* suffix, auto member) {
                std::cout << prefix;
                auto oldConfigValue = member(oldConfig);
                auto updatedConfigValue = member(updatedConfig);
                if (oldConfigValue == updatedConfigValue) {
                    std::cout << oldConfigValue;
                } else {
                    std::cout << oldConfigValue << " -> " << updatedConfigValue;
                }
                std::cout << suffix;
            };
            std::cout
                << "\n----------------------------------------------------"
                << "\nFilesystem configuration:"
                << "\n----------------------------------------------------";
            printValue("\n- Filesystem format version: ", "", [] (const CryConfig& config) {return config.Version(); });
            printValue("\n- Created with: CryFS ", "", [] (const CryConfig& config) { return config.CreatedWithVersion(); });
            printValue("\n- Last opened with: CryFS ", "", [] (const CryConfig& config) { return config.LastOpenedWithVersion(); });
            printValue("\n- Cipher: ", "", [] (const CryConfig& config) { return config.Cipher(); });
            printValue("\n- Blocksize: ", " bytes", [] (const CryConfig& config) { return config.BlocksizeBytes(); });
            printValue("\n- Filesystem Id: ", "", [] (const CryConfig& config) { return config.FilesystemId().ToString(); });
            std::cout << "\n----------------------------------------------------\n";
        }
    }

    void Cli::_runFilesystem(const ProgramOptions &options, std::function<void()> onMounted) {
        try {
            const LocalStateDir localStateDir(Environment::localStateDir());
            auto blockStore = make_unique_ref<OnDiskBlockStore2>(options.baseDir());
            auto config = _loadOrCreateConfig(options, localStateDir);
            printConfig(config.oldConfig, *config.configFile->config());
            unique_ptr<fspp::fuse::Fuse> fuse = nullptr;
            bool stoppedBecauseOfIntegrityViolation = false;

            auto onIntegrityViolation = [&fuse, &stoppedBecauseOfIntegrityViolation] () {
              if (fuse.get() != nullptr) {
                LOG(ERR, "Integrity violation detected. Unmounting.");
                stoppedBecauseOfIntegrityViolation = true;
                fuse->stop();
              } else {
                // Usually on an integrity violation, the file system is unmounted.
                // Here, the file system isn't initialized yet, i.e. we failed in the initial steps when
                // setting up _device before running initFilesystem.
                // We can't unmount a not-mounted file system, but we can make sure it doesn't get mounted.
                throw CryfsException("Integrity violation detected. Unmounting.", ErrorCode::IntegrityViolation);
              }
            };
            const bool missingBlockIsIntegrityViolation = config.configFile->config()->missingBlockIsIntegrityViolation();
            _device = optional<unique_ref<CryDevice>>(make_unique_ref<CryDevice>(std::move(config.configFile), std::move(blockStore), std::move(localStateDir), config.myClientId, options.allowIntegrityViolations(), missingBlockIsIntegrityViolation, std::move(onIntegrityViolation)));
            _sanityCheckFilesystem(_device->get());

            auto initFilesystem = [&] (fspp::fuse::Fuse *fs){
                ASSERT(_device != none, "File system not ready to be initialized. Was it already initialized before?");

                //TODO Test auto unmounting after idle timeout
                const boost::optional<double> idle_minutes = options.unmountAfterIdleMinutes();
                _idleUnmounter = _createIdleCallback(idle_minutes, [fs, idle_minutes] {
                    LOG(INFO, "Unmounting because file system was idle for {} minutes", *idle_minutes);
                    fs->stop();
                });
                if (_idleUnmounter != none) {
                    (*_device)->onFsAction(std::bind(&CallAfterTimeout::resetTimer, _idleUnmounter->get()));
                }

                return make_shared<fspp::FilesystemImpl>(std::move(*_device));
            };

            fuse = make_unique<fspp::fuse::Fuse>(initFilesystem, std::move(onMounted), "cryfs", "cryfs@" + options.baseDir().string());

            _initLogfile(options);

            std::cout << "\nMounting filesystem. To unmount, call:\n$ cryfs-unmount " << options.mountDir() << "\n"
                      << std::endl;

            if (options.foreground()) {
                fuse->runInForeground(options.mountDir(), options.fuseOptions());
            } else {
                fuse->runInBackground(options.mountDir(), options.fuseOptions());
            }

            if (stoppedBecauseOfIntegrityViolation) {
              throw CryfsException("Integrity violation detected. Unmounting.", ErrorCode::IntegrityViolation);
            }
        } catch (const CryfsException &e) {
            throw; // CryfsException is only thrown if setup goes wrong. Throw it through so that we get the correct process exit code.
        } catch (const std::exception &e) {
            LOG(ERR, "Crashed: {}", e.what());
        } catch (...) {
            LOG(ERR, "Crashed");
        }
    }

    void Cli::_sanityCheckFilesystem(CryDevice *device) {
        //Try to list contents of base directory
        auto _rootDir = device->Load("/"); // this might throw an exception if the root blob doesn't exist
        if (_rootDir == none) {
            throw CryfsException("Couldn't find root blob", ErrorCode::InvalidFilesystem);
        }
        auto rootDir = dynamic_pointer_move<CryDir>(*_rootDir);
        if (rootDir == none) {
            throw CryfsException("Base directory blob doesn't contain a directory", ErrorCode::InvalidFilesystem);
        }
        (*rootDir)->children(); // Load children
    }

    optional<unique_ref<CallAfterTimeout>> Cli::_createIdleCallback(optional<double> minutes, function<void()> callback) {
        if (minutes == none) {
            return none;
        }
        const uint64_t millis = std::llround(60000 * (*minutes));
        return make_unique_ref<CallAfterTimeout>(milliseconds(millis), callback, "idlecallback");
    }

    void Cli::_initLogfile(const ProgramOptions &options) {
        spdlog::drop("cryfs");
        //TODO Test that --logfile parameter works. Should be: file if specified, otherwise stderr if foreground, else syslog.
        if (options.logFile() != none) {
            cpputils::logging::setLogger(
                spdlog::create<spdlog::sinks::basic_file_sink_mt>("cryfs", options.logFile()->string()));
        } else if (options.foreground()) {
            cpputils::logging::setLogger(spdlog::stderr_logger_mt("cryfs"));
        } else {
            cpputils::logging::setLogger(cpputils::logging::system_logger("cryfs"));
        }
    }

	void Cli::_sanityChecks(const ProgramOptions &options) {
		_checkDirAccessible(bf::absolute(options.baseDir()), "base directory", options.createMissingBasedir(), ErrorCode::InaccessibleBaseDir);

		if (!options.mountDirIsDriveLetter()) {
			_checkDirAccessible(options.mountDir(), "mount directory", options.createMissingMountpoint(), ErrorCode::InaccessibleMountDir);
			_checkMountdirDoesntContainBasedir(options);
		} else {
			if (bf::exists(options.mountDir())) {
				throw CryfsException("Drive " + options.mountDir().string() + " already exists.", ErrorCode::InaccessibleMountDir);
			}
		}
    }

    void Cli::_checkDirAccessible(const bf::path &dir, const std::string &name, bool createMissingDir, ErrorCode errorCode) {
        if (!bf::exists(dir)) {
            bool create = createMissingDir;
            if (create) {
                LOG(INFO, "Automatically creating {}", name);
            } else {
                create = _console->askYesNo("Could not find " + name + ". Do you want to create it?", false);
            }
            if (create) {
                if (!bf::create_directory(dir)) {
                    throw CryfsException("Error creating "+name, errorCode);
                }
            } else {
                //std::cerr << "Exit code: " << exitCode(errorCode) << std::endl;
                throw CryfsException(name + " not found.", errorCode);
            }
        }
        if (!bf::is_directory(dir)) {
            throw CryfsException(name+" is not a directory.", errorCode);
        }
        auto file = _checkDirWriteable(dir, name, errorCode);
        _checkDirReadable(dir, file, name, errorCode);
    }

    shared_ptr<TempFile> Cli::_checkDirWriteable(const bf::path &dir, const std::string &name, ErrorCode errorCode) {
        auto path = dir / "tempfile";
        try {
            return make_shared<TempFile>(path);
        } catch (const std::runtime_error &e) {
            throw CryfsException("Could not write to "+name+".", errorCode);
        }
    }

    void Cli::_checkDirReadable(const bf::path &dir, shared_ptr<TempFile> tempfile, const std::string &name, ErrorCode errorCode) {
        ASSERT(bf::equivalent(dir, tempfile->path().parent_path()), "This function should be called with a file inside the directory");
        try {
            bool found = false;
            const bf::directory_iterator end;
            for (auto iter = bf::directory_iterator(dir); iter != end; ++iter) {
                if (bf::equivalent(*iter, tempfile->path())) {
                    found = true;
                }
            }
            if (!found) {
                //This should not happen. Can only happen if the written temp file got deleted inbetween or maybe was not written at all.
                throw std::runtime_error("Error accessing "+name+".");
            }
        } catch (const boost::filesystem::filesystem_error &e) {
            throw CryfsException("Could not read from "+name+".", errorCode);
        }
    }

    void Cli::_checkMountdirDoesntContainBasedir(const ProgramOptions &options) {
        if (_pathContains(options.mountDir(), options.baseDir())) {
            throw CryfsException("base directory can't be inside the mount directory.", ErrorCode::BaseDirInsideMountDir);
        }
    }

    bool Cli::_pathContains(const bf::path &parent, const bf::path &child) {
        const bf::path absParent = bf::canonical(parent);
        bf::path current = bf::canonical(child);
        if (absParent.empty() && current.empty()) {
            return true;
        }
        while(!current.empty()) {
            if (bf::equivalent(current, absParent)) {
                return true;
            }
            current = current.parent_path();
        }
        return false;
    }

    int Cli::main(int argc, const char **argv,
        #ifdef CRYFS_UPDATE_CHECKS
        unique_ref<HttpClient> httpClient,
        #endif
        std::function<void()> onMounted
    ) {
        cpputils::showBacktraceOnCrash();
        cpputils::set_thread_name("cryfs");

        try {
            _showVersion();
            #ifdef CRYFS_UPDATE_CHECKS
            _maybeCheckForUpdates(std::move(httpClient));
            #endif
            const ProgramOptions options = program_options::Parser(argc, argv).parse(CryCiphers::supportedCipherNames());
            _sanityChecks(options);
            _runFilesystem(options, std::move(onMounted));
        } catch (const CryfsException &e) {
            if (e.what() != string()) {
              std::cerr << "Error " << static_cast<int>(e.errorCode()) << ": " << e.what() << std::endl;
            }
            return exitCode(e.errorCode());
        } catch (const std::runtime_error &e) {
            std::cerr << "Error: " << e.what() << std::endl;
            return exitCode(ErrorCode::UnspecifiedError);
        }
        return exitCode(ErrorCode::Success);
    }
}
