#include "Cli.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cpp-utils/assert/backtrace.h>

#include <fspp/fuse/Fuse.h>
#include <fspp/impl/FilesystemImpl.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/io/DontEchoStdinToStdoutRAII.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <cryfs/impl/config/CryConfigLoader.h>
#include <cryfs/impl/config/CryPasswordBasedKeyProvider.h>
#include "program_options/Parser.h"
#include <boost/filesystem.hpp>

#include <cryfs/impl/filesystem/CryDir.h>
#include <gitversion/gitversion.h>

#include "VersionChecker.h"
#include <gitversion/VersionCompare.h>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <cryfs/impl/localstate/LocalStateDir.h>
#include <cryfs/impl/localstate/BasedirMetadata.h>
#include "Environment.h"
#include <cryfs/impl/CryfsException.h>
#include <cpp-utils/thread/debugging.h>

//TODO Many functions accessing the ProgramOptions object. Factor out into class that stores it as a member.
//TODO Factor out class handling askPassword

using namespace cryfs_cli;
using namespace cryfs;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

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

    Cli::Cli(RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, shared_ptr<Console> console):
            _keyGenerator(keyGenerator), _scryptSettings(scryptSettings), _console(), _noninteractive(false), _idleUnmounter(none), _device(none) {
        _noninteractive = Environment::isNoninteractive();
        if (_noninteractive) {
            _console = make_shared<NoninteractiveConsole>(console);
        } else {
            _console = console;
        }
    }

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
            bool again = false;
            do {
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
            } while (again);
            return password;
        };
    }

    bool Cli::_confirmPassword(cpputils::Console* console, const string &password) {
        string confirmPassword = console->askPassword("Confirm Password: ");
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

    void Cli::_runFilesystem(const ProgramOptions &options, std::function<void()> onMounted) {
        try {
            LocalStateDir localStateDir(Environment::localStateDir());
            auto config = _loadOrCreateConfig(options, localStateDir);
            printConfig(config.oldConfig, *config.configFile->config());
            unique_ptr<fspp::fuse::Fuse> fuse = nullptr;
            bool stoppedBecauseOfIntegrityViolation = false;

            auto onIntegrityViolation = [&fuse, &stoppedBecauseOfIntegrityViolation] () {
              if (fuse.get() != nullptr) {
                LOG(ERR, "Integrity violation detected after mounting. Unmounting.");
                stoppedBecauseOfIntegrityViolation = true;
                fuse->stop();
              } else {
                // Usually on an integrity violation, the file system is unmounted.
                // Here, the file system isn't initialized yet, i.e. we failed in the initial steps when
                // setting up _device before running initFilesystem.
                // We can't unmount a not-mounted file system, but we can make sure it doesn't get mounted.
                LOG(ERR, "Integrity violation detected before mounting. Not mounting.");
              }
            };
            const bool missingBlockIsIntegrityViolation = config.configFile->config()->missingBlockIsIntegrityViolation();
            _device = optional<unique_ref<CryDevice>>(make_unique_ref<CryDevice>(std::move(config.configFile), options.baseDir(), std::move(localStateDir), config.myClientId, options.allowIntegrityViolations(), missingBlockIsIntegrityViolation, std::move(onIntegrityViolation)));
            _sanityCheckFilesystem(_device->get());

            auto initFilesystem = [&] (fspp::fuse::Fuse *fs){
                ASSERT(_device != none, "File system not ready to be initialized. Was it already initialized before?");

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

    int Cli::main(int argc, const char **argv, unique_ref<HttpClient> httpClient, std::function<void()> onMounted) {
        cpputils::showBacktraceOnCrash();
        cpputils::set_thread_name("cryfs");

        try {
            _showVersion(std::move(httpClient));
            ProgramOptions options = program_options::Parser(argc, argv).parse(CryCiphers::supportedCipherNames());
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
