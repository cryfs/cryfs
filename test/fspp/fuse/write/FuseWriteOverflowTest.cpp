#include <cpp-utils/data/DataFixture.h>
#include "testutils/FuseWriteTest.h"
#include "../../testutils/InMemoryFile.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;
using ::testing::Invoke;
using ::testing::Action;

using cpputils::DataFixture;
using cpputils::Data;

using namespace fspp::fuse;

class FuseWriteOverflowTest: public FuseWriteTest {
public:
  size_t FILESIZE;
  size_t WRITESIZE;
  size_t OFFSET;

  WriteableInMemoryFile testFile;
  Data writeData;

  FuseWriteOverflowTest(size_t filesize, size_t writesize, size_t offset)
  : FILESIZE(filesize), WRITESIZE(writesize), OFFSET(offset), testFile(DataFixture::generate(FILESIZE)), writeData(DataFixture::generate(WRITESIZE)) {
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(fsimpl, write(0, _, _, _)).WillRepeatedly(WriteToFile);
  }

  // This write() mock implementation writes to the stored virtual file.
  Action<void(int, const void*, size_t, off_t)> WriteToFile =
    Invoke([this](int, const void *buf, size_t count, off_t offset) {
      testFile.write(buf, count, offset);
    });
};

class FuseWriteOverflowTestWithNonemptyFile: public FuseWriteOverflowTest {
public:
  FuseWriteOverflowTestWithNonemptyFile(): FuseWriteOverflowTest(1000, 2000, 500) {}
};

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteMoreThanFileSizeFromBeginning) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, 0);

  EXPECT_EQ(WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, 0));
}

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteMoreThanFileSizeFromMiddle) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, OFFSET);

  EXPECT_EQ(OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.regionUnchanged(0, OFFSET));
  EXPECT_TRUE(testFile.fileContentEquals(writeData, OFFSET));
}

TEST_F(FuseWriteOverflowTestWithNonemptyFile, WriteAfterFileEnd) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, FILESIZE + OFFSET);

  EXPECT_EQ(FILESIZE + OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.regionUnchanged(0, FILESIZE));
  EXPECT_TRUE(testFile.fileContentEquals(writeData, FILESIZE + OFFSET));
}

class FuseWriteOverflowTestWithEmptyFile: public FuseWriteOverflowTest {
public:
  FuseWriteOverflowTestWithEmptyFile(): FuseWriteOverflowTest(0, 2000, 500) {}
};

TEST_F(FuseWriteOverflowTestWithEmptyFile, WriteToBeginOfEmptyFile) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, 0);

  EXPECT_EQ(WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, 0));
}

TEST_F(FuseWriteOverflowTestWithEmptyFile, WriteAfterFileEnd) {
  WriteFile(FILENAME, writeData.data(), WRITESIZE, OFFSET);

  EXPECT_EQ(OFFSET + WRITESIZE, testFile.size());
  EXPECT_TRUE(testFile.fileContentEquals(writeData, OFFSET));
}
