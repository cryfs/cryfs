#include "testutils/FuseReadTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;

using namespace fspp::fuse;

class FuseReadFileDescriptorTest: public FuseReadTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseReadFileDescriptorTest, FuseReadFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseReadFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstatWithSize(FILENAME, fspp::num_bytes_t(1));
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(*fsimpl, read(Eq(GetParam()), _, _, _))
    .Times(1).WillOnce(ReturnSuccessfulRead);

  std::array<char, 1> buf{};
  ReadFile(FILENAME, buf.data(), fspp::num_bytes_t(1), fspp::num_bytes_t(0));
}
