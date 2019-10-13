#include "testutils/FuseFsyncTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Return;

using namespace fspp::fuse;

class FuseFsyncFileDescriptorTest: public FuseFsyncTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFsyncFileDescriptorTest, FuseFsyncFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseFsyncFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(*fsimpl, fsync(Eq(GetParam())))
    .Times(1).WillOnce(Return());

  FsyncFile(FILENAME);
}
