#include "testutils/FuseReadTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;


using namespace fspp::fuse;

class FuseReadOverflowTest: public FuseReadTest {
public:
  const size_t FILESIZE = 1000;
  const size_t READSIZE = 2000;
  const size_t OFFSET = 500;

  void SetUp() override {
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(fsimpl, read(0, _, _, _)).WillRepeatedly(ReturnSuccessfulReadRegardingSize(FILESIZE));
  }
};


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
