#include "../testutils/FuseTest.h"

#include <cpp-utils/tempfile/TempDir.h>

using fspp::fuse::Fuse;
using std::string;
using std::vector;

// Regression test for the mount error handling: Fuse::runInForeground() and Fuse::runInBackground()
// have to hand libfuse's exit code back to the caller. They used to discard it, which made a mount
// that libfuse refused look exactly like a file system that had been mounted and unmounted cleanly.
class FuseMountFailureTest: public FuseTest {
public:
  // libfuse parses 'entry_timeout' as a double, so it rejects this while setting up the file system.
  // The mount is refused before anything is mounted and fuse_main() returns a non-zero exit code.
  static vector<string> unparseableFuseOptions() {
    return {"-o", "entry_timeout=not_a_number"};
  }

  Fuse createFuse() {
    return Fuse([this] (Fuse*) {return fsimpl;}, []{}, "fusetest", boost::none);
  }
};

TEST_F(FuseMountFailureTest, WhenRunningInForeground_ThenLibfusesExitCodeIsReported) {
  const cpputils::TempDir mountDir;
  Fuse fuse = createFuse();

  const int exitCode = fuse.runInForeground(mountDir.path(), unparseableFuseOptions());

  EXPECT_NE(0, exitCode);
  EXPECT_FALSE(fuse.running());
}

TEST_F(FuseMountFailureTest, WhenRunningInBackground_ThenLibfusesExitCodeIsReported) {
  const cpputils::TempDir mountDir;
  Fuse fuse = createFuse();

  const int exitCode = fuse.runInBackground(mountDir.path(), unparseableFuseOptions());

  EXPECT_NE(0, exitCode);
  EXPECT_FALSE(fuse.running());
}
