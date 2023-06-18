#include "testutils/FuseFstatTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;

using namespace fspp::fuse;

// Cite from FUSE documentation on the fgetattr function:
// "Currently this is only called after the create() method if that is implemented (see above).
//  Later it may be called for invocations of fstat() too."
// So we need to issue a create to get our fstat called.

class FuseFstatParameterTest: public FuseFstatTest, public WithParamInterface<int> {
public:
  void CallFstat(const char *filename) {
    auto fs = TestFS();
    CreateFile(fs.get(), filename);
  }
};
INSTANTIATE_TEST_SUITE_P(FuseFstatParameterTest, FuseFstatParameterTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseFstatParameterTest, FileDescriptorIsCorrect) {
  ReturnDoesntExistOnLstat(FILENAME);
  OnCreateAndOpenReturnFileDescriptor(FILENAME, GetParam());

  EXPECT_CALL(*fsimpl, fstat(Eq(GetParam()), testing::_)).Times(1).WillOnce(ReturnIsFileFstat);

  CallFstat(FILENAME);
}
