#include "fspp/fs_interface/Types.h"
#include "testutils/FuseReadTest.h"

#include "gmock/gmock.h"
#include <array>
#include <gtest/gtest.h>

using namespace fspp::fuse;

class FuseReadOverflowTest: public FuseReadTest {
public:
  static constexpr fspp::num_bytes_t FILESIZE = fspp::num_bytes_t(1000);
  static constexpr fspp::num_bytes_t READSIZE = fspp::num_bytes_t(2000);
  static constexpr fspp::num_bytes_t OFFSET = fspp::num_bytes_t(500);

  void SetUp() override {
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(*fsimpl, read(0, testing::_, testing::_, testing::_)).WillRepeatedly(ReturnSuccessfulReadRegardingSize(FILESIZE));
  }
};

constexpr fspp::num_bytes_t FuseReadOverflowTest::FILESIZE;
constexpr fspp::num_bytes_t FuseReadOverflowTest::READSIZE;
constexpr fspp::num_bytes_t FuseReadOverflowTest::OFFSET;


TEST_F(FuseReadOverflowTest, ReadMoreThanFileSizeFromBeginning) {
  std::array<char, READSIZE.value()> buf{};
  auto retval = ReadFileReturnError(FILENAME, buf.data(), READSIZE, fspp::num_bytes_t(0));
  EXPECT_EQ(FILESIZE, retval.read_bytes);
}

TEST_F(FuseReadOverflowTest, ReadMoreThanFileSizeFromMiddle) {
  std::array<char, READSIZE.value()> buf{};
  auto retval = ReadFileReturnError(FILENAME, buf.data(), READSIZE, OFFSET);
  EXPECT_EQ(FILESIZE-OFFSET, retval.read_bytes);
}
