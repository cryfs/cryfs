#include "Cli.h"

#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <messmer/cpp-utils/assert/backtrace.h>
#include <messmer/cpp-utils/daemon/daemonize.h>

#include "messmer/fspp/fuse/Fuse.h"
#include "messmer/fspp/impl/FilesystemImpl.h"
#include "filesystem/CryDevice.h"
#include "config/CryConfigLoader.h"
#include "program_options/Parser.h"

#include <gitversion/version.h>

#include <pwd.h>

//<limits.h> needed for libc to define PASS_MAX
#include <limits.h>
#ifdef PASS_MAX
#error The used libc implementation has a maximal password size for getpass(). We cannot use it to ask for passwords.
#endif

using namespace cryfs;
namespace bf = boost::filesystem;

using blockstore::ondisk::OnDiskBlockStore;
using blockstore::inmemory::InMemoryBlockStore;
using program_options::ProgramOptions;

using cpputils::make_unique_ref;
using cpputils::Random;
using cpputils::IOStreamConsole;
using std::cout;
using std::string;
using std::endl;
using std::vector;
using boost::none;

//TODO Support files > 4GB
//TODO Improve parallelity.
//TODO Did deadlock in bonnie++ second run (in the create files sequentially) - maybe also in a later run or different step?
//TODO Improve error message when root blob wasn't found.
//TODO Replace ASSERTs with other error handling when it is not a programming error but an environment influence (e.g. a block is missing)
//TODO Fuse error messages like "fuse: bad mount point `...': Transport endpoint is not connected" go missing when running in background

namespace cryfs {

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
            auto &keyGenerator = Random::OSRandom();
            std::cout << "Loading config file..." << std::endl;
            auto config = CryConfigLoader(std::move(console), keyGenerator, &Cli::_askPassword).loadOrCreate(configFile);
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
        auto config = _loadOrCreateConfig(options);
        auto blockStore = make_unique_ref<OnDiskBlockStore>(bf::path(options.baseDir()));
        CryDevice device(std::move(config), std::move(blockStore));
        fspp::FilesystemImpl fsimpl(&device);
        fspp::fuse::Fuse fuse(&fsimpl);

        _goToBackgroundIfSpecified(options);

        vector<char *> fuseOptions = options.fuseOptions();
        std::cout << "\nFilesystem is running." << std::endl;
        fuse.run(fuseOptions.size(), fuseOptions.data());
    }

    void Cli::_goToBackgroundIfSpecified(const ProgramOptions &options) {
        if (!options.foreground()) {
            cpputils::daemonize();
            if (options.logFile() == none) {
                // Setup logging to syslog.
                cpputils::logging::setLogger(spdlog::syslog_logger("cryfs", "cryfs", LOG_PID));
            }
        }
    }

    int Cli::main(int argc, char *argv[]) {
        cpputils::showBacktraceOnSigSegv();
        _showVersion();

        ProgramOptions options = program_options::Parser(argc, argv).parse();
        //TODO Test that --logfile parameter works. Should be: file if specified, otherwise stderr if foreground, else syslog.
        if (options.logFile() != none) {
            cpputils::logging::setLogger(
                    spdlog::create<spdlog::sinks::simple_file_sink<std::mutex>>("cryfs", *options.logFile()));
        }
        _runFilesystem(options);
        return 0;
    }

}