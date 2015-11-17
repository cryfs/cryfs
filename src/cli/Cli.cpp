#include "Cli.h"

#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <messmer/cpp-utils/assert/backtrace.h>

#include "messmer/fspp/fuse/Fuse.h"
#include "messmer/fspp/impl/FilesystemImpl.h"
#include <messmer/cpp-utils/process/subprocess.h>
#include "../filesystem/CryDevice.h"
#include "../config/CryConfigLoader.h"
#include "program_options/Parser.h"

#include <gitversion/version.h>

#include <pwd.h>

//<limits.h> needed for libc to define PASS_MAX
#include <limits.h>
#ifdef PASS_MAX
#error The used libc implementation has a maximal password size for getpass(). We cannot use it to ask for passwords.
#endif

//TODO Many functions accessing the ProgramOptions object. Factor out into class that stores it as a member.

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
using std::cout;
using std::string;
using std::endl;
using std::vector;
using std::shared_ptr;
using std::make_shared;
using std::unique_ptr;
using std::make_unique;
using std::function;
using boost::optional;
using boost::none;
using boost::chrono::duration;
using boost::chrono::duration_cast;
using boost::chrono::minutes;
using boost::chrono::milliseconds;

//TODO Support files > 4GB
//TODO Improve parallelity.
//TODO Did deadlock in bonnie++ second run (in the create files sequentially) - maybe also in a later run or different step?
//TODO Replace ASSERTs with other error handling when it is not a programming error but an environment influence (e.g. a block is missing)

namespace cryfs {

    Cli::Cli(RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings):
            _keyGenerator(keyGenerator), _scryptSettings(scryptSettings) {}

    void Cli::_showVersion() {
        cout << "CryFS Version " << version::VERSION_STRING << endl;
        if (version::IS_DEV_VERSION) {
            cout << "WARNING! This is a development version based on git commit " << version::GIT_COMMIT_ID <<
            ". Please do not use in production!" << endl;
        } else if (!version::IS_STABLE_VERSION) {
            cout << "WARNING! This is an experimental version. Please backup your data frequently!" << endl;
        } else {
            //TODO This is shown for stable version numbers like 0.8 - remove once we reach 1.0
            cout << "WARNING! This version is not considered stable. Please backup your data frequently!" << endl;
        }
#ifndef NDEBUG
        cout << "WARNING! This is a debug build. Performance might be slow." << endl;
#endif
        cout << endl;
    }

    bool Cli::_checkPassword(const string &password) {
        if (password == "") {
            std::cerr << "Empty password not allowed. Please try again." << std::endl;
            return false;
        }
        return true;
    }

    string Cli::_getPassword(const ProgramOptions &options) {
        string password;
        if (options.extPass() == none) {
            password = _askPassword();
        } else {
            password = cpputils::Subprocess::call(*options.extPass());
        }
        //Remove trailing newline
        if (password[password.size()-1] == '\n') {
            password.resize(password.size()-1);
        }
        //Check that password is valid
        if (!_checkPassword(password)) {
            throw std::runtime_error("Password invalid.");
        }
        return password;
    }

    string Cli::_askPassword() {
        string password = getpass("Password: ");
        while (!_checkPassword(password)) {
            password = getpass("Password: ");
        }
        return password;
    };

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
            auto console = make_unique_ref<IOStreamConsole>();
            std::cout << "Loading config file..." << std::endl;
            auto config = CryConfigLoader(std::move(console), _keyGenerator, _scryptSettings, std::bind(&Cli::_getPassword, this, std::cref(options)), options.cipher()).loadOrCreate(configFile);
            std::cout << "Loading config file...done" << std::endl;
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

    void Cli::_runFilesystem(const ProgramOptions &options) {
        try {
            auto blockStore = make_unique_ref<OnDiskBlockStore>(bf::path(options.baseDir()));
            auto config = _loadOrCreateConfig(options);
            CryDevice device(std::move(config), std::move(blockStore));
            fspp::FilesystemImpl fsimpl(&device);
            fspp::fuse::Fuse fuse(&fsimpl);

            _initLogfile(options);

            //TODO Test auto unmounting after idle timeout
            //TODO This can fail due to a race condition if the filesystem isn't started yet (e.g. passing --unmount-idle 0").
            auto idleUnmounter = _createIdleCallback(options.unmountAfterIdleMinutes(), [&fuse] {fuse.stop();});
            if (idleUnmounter != none) {
                device.onFsAction(std::bind(&CallAfterTimeout::resetTimer, idleUnmounter->get()));
            }

            std::cout << "\nMounting filesystem. To unmount, call:\n$ fusermount -u " << options.mountDir() << "\n" << std::endl;

            vector<char *> fuseOptions = options.fuseOptions();
            fuse.run(fuseOptions.size(), fuseOptions.data());
        } catch (const std::exception &e) {
            LOG(ERROR) << "Crashed: " << e.what();
        } catch (...) {
            LOG(ERROR) << "Crashed";
        }
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
                spdlog::create<spdlog::sinks::simple_file_sink<std::mutex>>("cryfs", *options.logFile()));
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
            throw std::runtime_error(name+" not found.");
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

    int Cli::main(int argc, char *argv[]) {
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
