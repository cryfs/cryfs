#include "fspp/fs_interface/Types.h"
#include "testutils/FuseWriteTest.h"

#include "gmock/gmock.h"
#include "gtest/gtest.h"
#include <array>

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Return;

using namespace fspp::fuse;

class FuseWriteFileDescriptorTest: public FuseWriteTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseWriteFileDescriptorTest, FuseWriteFileDescriptorTest, Values(0,1,10,1000,1024*1024*1024));


TEST_P(FuseWriteFileDescriptorTest, FileDescriptorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  EXPECT_CALL(*fsimpl, write(Eq(GetParam()), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return());

  std::array<char, 1> buf{};
  WriteFile(FILENAME, buf.data(), fspp::num_bytes_t(1), fspp::num_bytes_t(0));
}
