#include "testutils/FuseReadTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;

using namespace fspp::fuse;

class FuseReadFileDescriptorTest: public FuseReadTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseReadFileDescriptorTest, FuseReadFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseReadFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstatWithSize(FILENAME, 1);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(fsimpl, read(Eq(GetParam()), _, _, _))
    .Times(1).WillOnce(ReturnSuccessfulRead);

  char buf[1];
  ReadFile(FILENAME, buf, 1, 0);
}
