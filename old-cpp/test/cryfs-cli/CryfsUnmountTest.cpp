#include "testutils/CliTest.h"
#include <cryfs-cli/Cli.h>
#include <cryfs-unmount/Cli.h>

using CliTest_Unmount = CliTest;
namespace bf = boost::filesystem;

namespace {
void unmount(const bf::path& mountdir) {
    std::vector<const char*> _args = {"cryfs-unmount", mountdir.string().c_str()};
    cryfs_unmount::Cli().main(2, _args.data());
}

TEST_F(CliTest_Unmount, givenMountedFilesystem_whenUnmounting_thenSucceeds) {
    // we're passing in boost::none as mountdir so EXPECT_RUN_SUCCESS doesn't unmount itself.
    // if the unmount we're calling here in the onMounted callback wouldn't work, EXPECT_RUN_SUCCESS
    // would never return and this would be a deadlock.
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "-f"}, boost::none, [this] () {
        unmount(mountdir);
    });
}

// TODO Test calling with invalid args, valid '--version' or '--help' args, with a non-mounted mountdir and a nonexisting mountdir.

}
