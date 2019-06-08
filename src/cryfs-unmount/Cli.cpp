#include "Cli.h"
#include <fspp/fuse/Fuse.h>
#include <cryfs-unmount/program_options/Parser.h>
#include <gitversion/gitversion.h>
#include <cryfs/CryfsException.h>

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
    ProgramOptions options = Parser(argc, argv).parse();

    if (!boost::filesystem::exists(options.mountDir())) {
        throw cryfs::CryfsException("Given mountdir doesn't exist", cryfs::ErrorCode::InaccessibleMountDir);
    }
    // TODO This doesn't seem to work with relative paths
    std::cout << "Unmounting CryFS filesystem at " << options.mountDir() << "." << std::endl;
    Fuse::unmount(options.mountDir());

    // TODO Wait until it is actually unmounted and then show a better success message?
    std::cout << "Filesystem is unmounting now." << std::endl;
}

}
