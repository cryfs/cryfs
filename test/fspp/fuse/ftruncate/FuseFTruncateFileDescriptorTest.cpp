#include "testutils/FuseFTruncateTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Return;

using namespace fspp::fuse;

class FuseFTruncateFileDescriptorTest: public FuseFTruncateTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseFTruncateFileDescriptorTest, FuseFTruncateFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseFTruncateFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(fsimpl, ftruncate(Eq(GetParam()), _))
    .Times(1).WillOnce(Return());
  //Needed to make ::ftruncate system call return successfully
  ReturnIsFileOnFstat(GetParam());

  FTruncateFile(FILENAME, 0);
}
