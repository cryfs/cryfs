#include "testutils/FuseReadTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::_;


using namespace fspp::fuse;

class FuseReadOverflowTest: public FuseReadTest {
public:
  static constexpr size_t FILESIZE = 1000;
  static constexpr size_t READSIZE = 2000;
  static constexpr size_t OFFSET = 500;

  void SetUp() override {
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(fsimpl, read(0, _, _, _)).WillRepeatedly(ReturnSuccessfulReadRegardingSize(FILESIZE));
  }
};

constexpr size_t FuseReadOverflowTest::FILESIZE;
constexpr size_t FuseReadOverflowTest::READSIZE;
constexpr size_t FuseReadOverflowTest::OFFSET;


TEST_F(FuseReadOverflowTest, ReadMoreThanFileSizeFromBeginning) {
  char buf[READSIZE];
  auto retval = ReadFileReturnError(FILENAME, buf, READSIZE, 0);
  EXPECT_EQ(FILESIZE, retval.read_bytes);
}

TEST_F(FuseReadOverflowTest, ReadMoreThanFileSizeFromMiddle) {
  char buf[READSIZE];
  auto retval = ReadFileReturnError(FILENAME, buf, READSIZE, OFFSET);
  EXPECT_EQ(FILESIZE-OFFSET, retval.read_bytes);
}
