#include "testutils/CliTest.h"

using cryfs::ErrorCode;
using std::string;
using std::vector;

// Tests that a mount libfuse refuses is reported as an error instead of a successful run.
class CliTest_MountFailure: public CliTest {
public:
    // libfuse parses 'entry_timeout' as a double, so it rejects this while setting up the file
    // system and refuses the mount before anything gets mounted.
    vector<string> argsWithUnparseableFuseOption() {
        return {basedir.string(), mountdir.string(), "-f", "--cipher", "aes-256-gcm",
                "-o", "entry_timeout=not_a_number"};
    }
};

TEST_F(CliTest_MountFailure, WhenLibfuseRefusesToMount_ThenCryfsExitsWithAnError) {
    EXPECT_RUN_ERROR(
        argsWithUnparseableFuseOption(),
        "Error 26: Failed to mount filesystem",
        ErrorCode::MountFailed
    );
}
