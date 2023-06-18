#include <cpp-utils/data/DataFixture.h>
#include "testutils/FuseWriteTest.h"
#include "../../testutils/InMemoryFile.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Invoke;
using ::testing::Action;

using cpputils::DataFixture;
using cpputils::Data;

using namespace fspp::fuse;

class FuseWriteOverflowTest: public FuseWriteTest {
public:
  fspp::num_bytes_t FILESIZE;
  fspp::num_bytes_t WRITESIZE;
  fspp::num_bytes_t OFFSET;

  WriteableInMemoryFile testFile;
  Data writeData;

  FuseWriteOverflowTest(fspp::num_bytes_t filesize, fspp::num_bytes_t writesize, fspp::num_bytes_t offset)
  : FILESIZE(filesize), WRITESIZE(writesize), OFFSET(offset), testFile(DataFixture::generate(FILESIZE.value())), writeData(DataFixture::generate(WRITESIZE.value())) {
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(*fsimpl, write(0, testing::_, testing::_, testing::_)).WillRepeatedly(WriteToFile);
  }

  // This write() mock implementation writes to the stored virtual file.
  Action<void(int, const void*, fspp::num_bytes_t, fspp::num_bytes_t)> WriteToFile =
    Invoke([this](int, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
      testFile.write(buf, count, offset);
    });
};

class FuseWriteOverflowTestWithNonemptyFile: public FuseWriteOverflowTest {
public:
  FuseWriteOverflowTestWithNonemptyFile(): FuseWriteOverflowTest(fspp::num_bytes_t(1000), fspp::num_bytes_t(2000), fspp::num_bytes_t(500)) {}
};

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteMoreThanFileSizeFromBeginning) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, fspp::num_bytes_t(0));

  EXPECT_EQ(WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, fspp::num_bytes_t(0)));
}

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteMoreThanFileSizeFromMiddle) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, OFFSET);

  EXPECT_EQ(OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.regionUnchanged(fspp::num_bytes_t(0), OFFSET));
  EXPECT_TRUE(testFile.fileContentEquals(writeData, OFFSET));
}

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteAfterFileEnd) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, FILESIZE + OFFSET);

  EXPECT_EQ(FILESIZE + OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.regionUnchanged(fspp::num_bytes_t(0), FILESIZE));
  EXPECT_TRUE(testFile.fileContentEquals(writeData, FILESIZE + OFFSET));
}

class FuseWriteOverflowTestWithEmptyFile: public FuseWriteOverflowTest {
public:
  FuseWriteOverflowTestWithEmptyFile(): FuseWriteOverflowTest(fspp::num_bytes_t(0), fspp::num_bytes_t(2000), fspp::num_bytes_t(500)) {}
};

TEST_F(FuseWriteOverflowTestWithEmptyFile, WriteToBeginOfEmptyFile) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, fspp::num_bytes_t(0));

  EXPECT_EQ(WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, fspp::num_bytes_t(0)));
}

TEST_F(FuseWriteOverflowTestWithEmptyFile, WriteAfterFileEnd) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, OFFSET);

  EXPECT_EQ(OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, OFFSET));
}
