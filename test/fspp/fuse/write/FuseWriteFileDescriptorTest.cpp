#include "testutils/FuseWriteTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Return;

using namespace fspp::fuse;

class FuseWriteFileDescriptorTest: public FuseWriteTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseWriteFileDescriptorTest, FuseWriteFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseWriteFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(*fsimpl, write(Eq(GetParam()), _, _, _))
    .Times(1).WillOnce(Return());

  std::array<char, 1> buf{};
  WriteFile(FILENAME, buf.data(), fspp::num_bytes_t(1), fspp::num_bytes_t(0));
}
