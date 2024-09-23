#include "Cli.h"
#include "cryfs-unmount/program_options/ProgramOptions.h"
#include "cryfs/impl/ErrorCodes.h"
#include "gitversion/versionstring.h"
#include <boost/filesystem/operations.hpp>
#include <cryfs-unmount/program_options/Parser.h>
#include <cryfs/impl/CryfsException.h>
#include <fspp/fuse/Fuse.h>

#include <iostream>

using fspp::fuse::Fuse;
using cryfs_unmount::program_options::Parser;
using cryfs_unmount::program_options::ProgramOptions;

namespace cryfs_unmount {

namespace {
void _showVersion() {
    std::cout << "CryFS Version " << gitversion::VersionString() << std::endl;
}
}

void Cli::main(int argc, const char **argv) {
    _showVersion();
    const ProgramOptions options = Parser(argc, argv).parse();

    if (!boost::filesystem::exists(options.mountDir())) {
        throw cryfs::CryfsException("Given mountdir doesn't exist", cryfs::ErrorCode::InaccessibleMountDir);
    }

    bool immediate = options.immediate(); // NOLINT(misc-const-correctness) -- this cannot be const because it is modified in a platform-specific ifdef below
#if defined(__APPLE__)
    if (options.immediate()) {
        std::cerr << "Warning: OSX doesn't support the --immediate flag. Ignoring it.";
        immediate = false;
    }
#elif defined(_MSC_VER)
    if (options.immediate()) {
        std::cerr << "Warning: Windows doesn't support the --immediate flag. Ignoring it.";
        immediate = false;
    }
#endif

    // TODO This doesn't seem to work with relative paths
    std::cout << "Unmounting CryFS filesystem at " << options.mountDir() << "." << std::endl;
    if (immediate) {
        Fuse::unmount(options.mountDir(), true);

        // TODO Wait until it is actually unmounted and then show a better success message?
        std::cout << "Filesystem is unmounting." << std::endl;
    } else {
        Fuse::unmount(options.mountDir(), false);

        // TODO Wait until it is actually unmounted and then show a better success message?
        std::cout << "Filesystem will unmount as soon as nothing is accessing it anymore." << std::endl;
    }
}

}
