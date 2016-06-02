#include "Cli.h"

#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cpp-utils/assert/backtrace.h>

#include <fspp/fuse/Fuse.h>
#include <fspp/impl/FilesystemImpl.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/io/DontEchoStdinToStdoutRAII.h>
#include <cryfs/filesystem/CryDevice.h>
#include <cryfs/config/CryConfigLoader.h>
#include "program_options/Parser.h"
#include <boost/filesystem.hpp>

#include <cryfs/filesystem/CryDir.h>
#include <gitversion/gitversion.h>

#include "VersionChecker.h"
#include <gitversion/VersionCompare.h>
#include "Environment.h"

//TODO Fails with gpg-homedir in filesystem: gpg --homedir gpg-homedir --gen-key
//TODO Many functions accessing the ProgramOptions object. Factor out into class that stores it as a member.
//TODO Factor out class handling askPassword

using namespace cryfs;
namespace bf = boost::filesystem;
using namespace cpputils::logging;

using blockstore::ondisk::OnDiskBlockStore;
using blockstore::inmemory::InMemoryBlockStore;
using program_options::ProgramOptions;

using cpputils::make_unique_ref;
using cpputils::Random;
using cpputils::IOStreamConsole;
using cpputils::TempFile;
using cpputils::RandomGenerator;
using cpputils::unique_ref;
using cpputils::SCryptSettings;
using cpputils::Console;
using cpputils::HttpClient;
using cpputils::DontEchoStdinToStdoutRAII;
using std::cin;
using std::cout;
using std::string;
using std::endl;
using std::vector;
using std::shared_ptr;
using std::make_shared;
using std::unique_ptr;
using std::make_unique;
using std::function;
using std::make_shared;
using boost::optional;
using boost::none;
using boost::chrono::duration;
using boost::chrono::duration_cast;
using boost::chrono::minutes;
using boost::chrono::milliseconds;
using cpputils::dynamic_pointer_move;
using gitversion::VersionCompare;

//TODO Delete a large file in parallel possible? Takes a long time right now...
//TODO Improve parallelity.
//TODO Replace ASSERTs with other error handling when it is not a programming error but an environment influence (e.g. a block is missing)
//TODO Can we improve performance by setting compiler parameter -maes for scrypt?
//TODO Running nano in a cryfs file system, editing and saving an existing file shows "file was modified since opening".

namespace cryfs {

    Cli::Cli(RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, shared_ptr<Console> console, shared_ptr<HttpClient> httpClient):
            _keyGenerator(keyGenerator), _scryptSettings(scryptSettings), _console(console), _httpClient(httpClient), _noninteractive(false) {
        _noninteractive = Environment::isNoninteractive();
    }

    void Cli::_showVersion() {
        cout << "CryFS Version " << gitversion::VersionString() << endl;
        if (gitversion::IsDevVersion()) {
            cout << "WARNING! This is a development version based on git commit " << gitversion::GitCommitId() <<
            ". Please do not use in production!" << endl;
        } else if (!gitversion::IsStableVersion()) {
            cout << "WARNING! This is an experimental version. Please backup your data frequently!" << endl;
        } else {
            //TODO This is even shown for stable version numbers like 0.8 - remove once we reach 1.0
            cout << "WARNING! This version is not considered stable. Please backup your data frequently!" << endl;
        }
#ifndef NDEBUG
        cout << "WARNING! This is a debug build. Performance might be slow." << endl;
#endif
        if (!Environment::noUpdateCheck()) {
            _checkForUpdates();
        } else {
            cout << "Automatic checking for security vulnerabilities and updates is disabled." << endl;
        }
        cout << endl;
    }

    void Cli::_checkForUpdates() {
        VersionChecker versionChecker(_httpClient);
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

    bool Cli::_checkPassword(const string &password) {
        if (password == "") {
            std::cerr << "Empty password not allowed. Please try again." << std::endl;
            return false;
        }
        return true;
    }

    string Cli::_askPasswordForExistingFilesystem() {
        string password = _askPasswordFromStdin("Password: ");
        while (!_checkPassword(password)) {
            password = _askPasswordFromStdin("Password: ");
        }
        return password;
    };

    string Cli::_askPasswordForNewFilesystem() {
        string password;
        bool again = false;
        do {
            password = _askPasswordFromStdin("Password: ");
            if (!_checkPassword(password)) {
                again = true;
                continue;
            }
            if (!_confirmPassword(password)) {
                again = true;
                continue;
            }
            again = false;
        } while(again);
        return password;
    }

    bool Cli::_confirmPassword(const string &password) {
        string confirmPassword = _askPasswordFromStdin("Confirm Password: ");
        if (password != confirmPassword) {
            std::cout << "Passwords don't match" << std::endl;
            return false;
        }
        return true;
    }

    string Cli::_askPasswordNoninteractive() {
        //TODO Test
        string password = _askPasswordFromStdin("Password: ");
        if (!_checkPassword(password)) {
            throw std::runtime_error("Invalid password. Password cannot be empty.");
        }
        return password;
    }

    string Cli::_askPasswordFromStdin(const string &prompt) {
        DontEchoStdinToStdoutRAII _stdin_input_is_hidden_as_long_as_this_is_in_scope;

        std::cout << prompt << std::flush;
        string result;
        std::getline(cin, result);
        std::cout << std::endl;

        //Remove trailing newline
        if (result[result.size()-1] == '\n') {
            result.resize(result.size()-1);
        }

        return result;
    }

    bf::path Cli::_determineConfigFile(const ProgramOptions &options) {
        auto configFile = options.configFile();
        if (configFile == none) {
            return bf::path(options.baseDir()) / "cryfs.config";
        }
        return *configFile;
    }

    CryConfigFile Cli::_loadOrCreateConfig(const ProgramOptions &options) {
        try {
            auto configFile = _determineConfigFile(options);
            auto config = _loadOrCreateConfigFile(configFile, options.cipher(), options.blocksizeBytes());
            if (config == none) {
                std::cerr << "Could not load config file. Did you enter the correct password?" << std::endl;
                exit(1);
            }
            return std::move(*config);
        } catch (const std::exception &e) {
            std::cerr << "Error: " << e.what() << std::endl;
            exit(1);
        }
    }

    optional<CryConfigFile> Cli::_loadOrCreateConfigFile(const bf::path &configFilePath, const optional<string> &cipher, const optional<uint32_t> &blocksizeBytes) {
        if (_noninteractive) {
            return CryConfigLoader(_console, _keyGenerator, _scryptSettings,
                                   &Cli::_askPasswordNoninteractive,
                                   &Cli::_askPasswordNoninteractive,
                                   cipher, blocksizeBytes, _noninteractive).loadOrCreate(configFilePath);
        } else {
            return CryConfigLoader(_console, _keyGenerator, _scryptSettings,
                                   &Cli::_askPasswordForExistingFilesystem,
                                   &Cli::_askPasswordForNewFilesystem,
                                   cipher, blocksizeBytes, _noninteractive).loadOrCreate(configFilePath);
        }
    }

    void Cli::_runFilesystem(const ProgramOptions &options) {
        try {
            auto blockStore = make_unique_ref<OnDiskBlockStore>(options.baseDir());
            auto config = _loadOrCreateConfig(options);
            CryDevice device(std::move(config), std::move(blockStore));
            _sanityCheckFilesystem(&device);
            fspp::FilesystemImpl fsimpl(&device);
            fspp::fuse::Fuse fuse(&fsimpl, "cryfs", "cryfs@"+options.baseDir().native());

            _initLogfile(options);

            //TODO Test auto unmounting after idle timeout
            //TODO This can fail due to a race condition if the filesystem isn't started yet (e.g. passing --unmount-idle 0").
            auto idleUnmounter = _createIdleCallback(options.unmountAfterIdleMinutes(), [&fuse] {fuse.stop();});
            if (idleUnmounter != none) {
                device.onFsAction(std::bind(&CallAfterTimeout::resetTimer, idleUnmounter->get()));
            }

#ifdef __APPLE__
            std::cout << "\nMounting filesystem. To unmount, call:\n$ umount " << options.mountDir() << "\n" << std::endl;
#else
            std::cout << "\nMounting filesystem. To unmount, call:\n$ fusermount -u " << options.mountDir() << "\n" << std::endl;
#endif
            fuse.run(options.mountDir(), options.fuseOptions());
        } catch (const std::exception &e) {
            LOG(ERROR) << "Crashed: " << e.what();
        } catch (...) {
            LOG(ERROR) << "Crashed";
        }
    }

    void Cli::_sanityCheckFilesystem(CryDevice *device) {
        //Try to list contents of base directory
        auto _rootDir = device->Load("/"); // this might throw an exception if the root blob doesn't exist
        if (_rootDir == none) {
            throw std::runtime_error("Couldn't find root blob");
        }
        auto rootDir = dynamic_pointer_move<CryDir>(*_rootDir);
        if (rootDir == none) {
            throw std::runtime_error("Base directory blob doesn't contain a directory");
        }
        (*rootDir)->children(); // Load children
    }

    optional<unique_ref<CallAfterTimeout>> Cli::_createIdleCallback(optional<double> minutes, function<void()> callback) {
        if (minutes == none) {
            return none;
        }
        uint64_t millis = std::round(60000 * (*minutes));
        return make_unique_ref<CallAfterTimeout>(milliseconds(millis), callback);
    }

    void Cli::_initLogfile(const ProgramOptions &options) {
        spdlog::drop("cryfs");
        //TODO Test that --logfile parameter works. Should be: file if specified, otherwise stderr if foreground, else syslog.
        if (options.logFile() != none) {
            cpputils::logging::setLogger(
                spdlog::create<spdlog::sinks::simple_file_sink<std::mutex>>("cryfs", options.logFile()->native()));
        } else if (options.foreground()) {
            cpputils::logging::setLogger(spdlog::stderr_logger_mt("cryfs"));
        } else {
            cpputils::logging::setLogger(spdlog::syslog_logger("cryfs", "cryfs", LOG_PID));
        }
    }

    void Cli::_sanityChecks(const ProgramOptions &options) {
        _checkDirAccessible(options.baseDir(), "base directory");
        _checkDirAccessible(options.mountDir(), "mount directory");
        _checkMountdirDoesntContainBasedir(options);
    }

    void Cli::_checkDirAccessible(const bf::path &dir, const std::string &name) {
        if (!bf::exists(dir)) {
            if (_noninteractive) {
                //If we use the noninteractive frontend, don't ask whether to create the directory, but just fail.
                throw std::runtime_error(name + " not found");
            }
            bool create = _console->askYesNo("Could not find " + name + ". Do you want to create it?");
            if (create) {
                if (!bf::create_directory(dir)) {
                    throw std::runtime_error("Error creating "+name);
                }
            } else {
                throw std::runtime_error(name + " not found.");
            }
        }
        if (!bf::is_directory(dir)) {
            throw std::runtime_error(name+" is not a directory.");
        }
        auto file = _checkDirWriteable(dir, name);
        _checkDirReadable(dir, file, name);
    }

    shared_ptr<TempFile> Cli::_checkDirWriteable(const bf::path &dir, const std::string &name) {
        auto path = dir / "tempfile";
        try {
            return make_shared<TempFile>(path);
        } catch (const std::runtime_error &e) {
            throw std::runtime_error("Could not write to "+name+".");
        }
    }

    void Cli::_checkDirReadable(const bf::path &dir, shared_ptr<TempFile> tempfile, const std::string &name) {
        ASSERT(bf::equivalent(dir, tempfile->path().parent_path()), "This function should be called with a file inside the directory");
        try {
            bool found = false;
            bf::directory_iterator end;
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
            throw std::runtime_error("Could not read from "+name+".");
        }
    }

    void Cli::_checkMountdirDoesntContainBasedir(const ProgramOptions &options) {
        if (_pathContains(options.mountDir(), options.baseDir())) {
            throw std::runtime_error("base directory can't be inside the mount directory.");
        }
    }

    bool Cli::_pathContains(const bf::path &parent, const bf::path &child) {
        bf::path absParent = bf::canonical(parent);
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

    int Cli::main(int argc, const char *argv[]) {
        cpputils::showBacktraceOnSigSegv();
        _showVersion();

        ProgramOptions options = program_options::Parser(argc, argv).parse(CryCiphers::supportedCipherNames());

        try {
            _sanityChecks(options);
            _runFilesystem(options);
        } catch (const std::runtime_error &e) {
            std::cerr << "Error: " << e.what() << std::endl;
            exit(1);
        }
        return 0;
    }
}
