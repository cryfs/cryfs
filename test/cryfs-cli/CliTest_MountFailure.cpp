#include "testutils/CliTest.h"

using cryfs::ErrorCode;
using std::string;
using std::vector;

// Tests that a mount libfuse refuses is reported as an error instead of a successful run.
class CliTest_MountFailure: public CliTest {
public:
    // libfuse parses 'entry_timeout' as a double, so it rejects this while setting up the file
    // system and refuses the mount before anything gets mounted. Dokany's FUSE wrapper on Windows
    // doesn't know 'entry_timeout' and silently ignores options it doesn't know, but it parses
    // 'daemon_timeout' as an integer and rejects this the same way.
    vector<string> argsWithUnparseableFuseOption() {
#if defined(_MSC_VER)
        const char* option = "daemon_timeout=not_a_number";
#else
        const char* option = "entry_timeout=not_a_number";
#endif
        return {basedir.string(), mountpoint.string(), "-f", "--cipher", "aes-256-gcm",
                "-o", option};
    }
};

TEST_F(CliTest_MountFailure, WhenLibfuseRefusesToMount_ThenCryfsExitsWithAnError) {
    EXPECT_RUN_ERROR(
        argsWithUnparseableFuseOption(),
        "Error 26: Failed to mount filesystem",
        ErrorCode::MountFailed
    );
}
