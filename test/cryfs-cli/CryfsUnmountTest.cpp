#include "testutils/CliTest.h"
#include <cryfs-cli/Cli.h>
#include <cryfs-unmount/Cli.h>

using CliTest_Unmount = CliTest;
namespace bf = boost::filesystem;

namespace {
void unmount(const bf::path& mountdir) {
    // On Windows, path::string() returns a copy, so it has to be kept alive while the arguments
    // point into it. On Linux and macOS it returns a reference to the path's own string, which is
    // why the temporary this used to point into went unnoticed there.
    const std::string mountdir_string = mountdir.string();
    std::vector<const char*> _args = {"cryfs-unmount", mountdir_string.c_str()};
    cryfs_unmount::Cli().main(2, _args.data());
}

TEST_F(CliTest_Unmount, givenMountedFilesystem_whenUnmounting_thenSucceeds) {
    // This test unmounts itself, so EXPECT_RUN_SUCCESS must not do it as well.
    // if the unmount we're calling here in the onMounted callback wouldn't work, EXPECT_RUN_SUCCESS
    // would never return and this would be a deadlock.
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountpoint.string().c_str(), "-f"}, mountpoint, [this] () {
        unmount(mountpoint);
    }, UnmountAfterwards::No);
}

// TODO Test calling with invalid args, valid '--version' or '--help' args, with a non-mounted mountdir and a nonexisting mountdir.

}
