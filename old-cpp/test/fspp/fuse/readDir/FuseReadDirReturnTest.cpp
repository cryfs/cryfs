#include "testutils/FuseReadDirTest.h"
#include <cpp-utils/pointer/unique_ref.h>
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::vector;
using std::string;

using namespace fspp::fuse;

vector<string> LARGE_DIR(int num_entries) {
  vector<string> result;
  result.reserve(num_entries);
  for(int i=0; i<num_entries; ++i) {
    result.push_back("File "+std::to_string(i)+" file");
  }
  return result;
}

class FuseReadDirReturnTest: public FuseReadDirTest, public WithParamInterface<vector<string>> {
public:
  void testDirEntriesAreCorrect(const vector<string> &direntries) {
    ReturnIsDirOnLstat(DIRNAME);
    EXPECT_CALL(*fsimpl, readDir(Eq(DIRNAME)))
      .Times(1).WillOnce(ReturnDirEntries(direntries));

    auto returned_dir_entries = ReadDir(DIRNAME);
    EXPECT_EQ(direntries, returned_dir_entries);
  }
};
INSTANTIATE_TEST_SUITE_P(FuseReadDirReturnTest, FuseReadDirReturnTest, Values(
    vector<string>({}),
    vector<string>({"oneentry"}),
    vector<string>({"twoentries_1", "twoentries_2"}),
    vector<string>({"file1", "file with spaces"}),
    vector<string>({"file1", ".dotfile"})
));

TEST_P(FuseReadDirReturnTest, ReturnedDirEntriesAreCorrect) {
  testDirEntriesAreCorrect(GetParam());
}

// If using this with GTest Value-Parametrized TEST_P, it breaks some other unrelated tests
// (probably because it is doing a lot of construction work on the start of the test program)
TEST_F(FuseReadDirReturnTest, ReturnedDirEntriesAreCorrect_LargeDir1000) {
  auto direntries = LARGE_DIR(1000);
  testDirEntriesAreCorrect(direntries);
}

// If using this with GTest Value-Parametrized TEST_P, it breaks some other unrelated tests
// (probably because it is doing a lot of construction work on the start of the test program)
// DISABLED, because it uses a lot of memory
TEST_F(FuseReadDirReturnTest, DISABLED_ReturnedDirEntriesAreCorrect_LargeDir1000000) {
  auto direntries = LARGE_DIR(1000000);
  testDirEntriesAreCorrect(direntries);
}
