#include "testutils/CliTest.h"

#include <boost/filesystem.hpp>
#include <fstream>

namespace bf = boost::filesystem;
using cryfs::ErrorCode;
using std::string;
using std::vector;

// libfuse 2 refused to mount over a directory that already had files in it unless '-o nonempty' was
// given. libfuse 3.0 removed that check together with the option, so CryFS has to do it itself.
class CliTest_NonEmptyMountdir: public CliTest {
public:
    void PutFileIntoMountdir() {
        std::ofstream file((mountdir / "important.txt").string());
        file << "please don't hide me" << std::endl;
    }

    bool MountdirIsEmpty() {
        const bf::directory_iterator end;
        return end == bf::directory_iterator(mountdir);
    }

    vector<string> args(const vector<string>& extraArgs = {}) {
        vector<string> result = {basedir.string(), mountdir.string(), "-f", "--cipher", "aes-256-gcm"};
        result.insert(result.end(), extraArgs.begin(), extraArgs.end());
        return result;
    }
};

TEST_F(CliTest_NonEmptyMountdir, WhenMountdirIsNotEmpty_ThenMountingIsRefused) {
    PutFileIntoMountdir();
    EXPECT_RUN_ERROR(
        args(),
        "Error 17: mount directory is not empty",
        ErrorCode::InaccessibleMountDir
    );
}

TEST_F(CliTest_NonEmptyMountdir, WhenMountdirIsNotEmptyAndNonemptyOptionIsGiven_ThenMountingSucceeds) {
    PutFileIntoMountdir();
    bool wasHiddenWhileMounted = false;
    EXPECT_RUN_SUCCESS(args({"-o", "nonempty"}), mountdir, [&] {
        wasHiddenWhileMounted = MountdirIsEmpty();
    });
    EXPECT_TRUE(wasHiddenWhileMounted) << "The file in the mount directory should be hidden while mounted.";
}

// Counter-test: an empty mount directory keeps working without the option.
TEST_F(CliTest_NonEmptyMountdir, WhenMountdirIsEmpty_ThenMountingSucceeds) {
    ASSERT_TRUE(MountdirIsEmpty());
    EXPECT_RUN_SUCCESS(args(), mountdir);
}
