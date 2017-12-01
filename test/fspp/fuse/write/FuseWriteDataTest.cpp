#include <cpp-utils/data/DataFixture.h>
#include "testutils/FuseWriteTest.h"
#include "../../testutils/InMemoryFile.h"

#include "fspp/fuse/FuseErrnoException.h"

#include <tuple>
#include <cstdlib>

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Combine;
using ::testing::Invoke;
using ::testing::Action;

using std::tuple;
using std::get;
using cpputils::Data;
using cpputils::DataFixture;

using namespace fspp::fuse;

// We can't test the count or size parameter directly, because fuse doesn't pass them 1:1.
// But we can test that the data passed to the ::write syscall is correctly written.

struct TestData {
  TestData(): count(0), offset(0), additional_bytes_at_end_of_file(0) {}
  TestData(const tuple<size_t, off_t, size_t> &data): count(get<0>(data)), offset(get<1>(data)), additional_bytes_at_end_of_file(get<2>(data)) {}
  size_t count;
  off_t offset;
  //How many more bytes does the file have after the read block?
  size_t additional_bytes_at_end_of_file;
  size_t fileSize() {
    return count + offset + additional_bytes_at_end_of_file;
  }
};

// The testcase creates random data in memory, offers a mock write() implementation to write to this
// memory region and check methods to check for data equality of a region.
class FuseWriteDataTest: public FuseWriteTest, public WithParamInterface<tuple<size_t, off_t, size_t>> {
public:
  std::unique_ptr<WriteableInMemoryFile> testFile;
  TestData testData;

  FuseWriteDataTest()
          : testFile(nullptr),
            testData(GetParam()) {
    testFile = std::make_unique<WriteableInMemoryFile>(DataFixture::generate(testData.fileSize(), 1));
    ReturnIsFileOnLstatWithSize(FILENAME, testData.fileSize());
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(fsimpl, write(0, _, _, _))
      .WillRepeatedly(WriteToFile);
  }

  // This write() mock implementation writes to the stored virtual file.
  Action<void(int, const void*, size_t, off_t)> WriteToFile = Invoke([this](int, const void *buf, size_t count, off_t offset) {
    testFile->write(buf, count, offset);
  });
};
INSTANTIATE_TEST_CASE_P(FuseWriteDataTest, FuseWriteDataTest, Combine(Values(0,1,10,1000,1024, 10*1024*1024), Values(0, 1, 10, 1024, 10*1024*1024), Values(0, 1, 10, 1024, 10*1024*1024)));


TEST_P(FuseWriteDataTest, DataWasCorrectlyWritten) {
  Data randomWriteData = DataFixture::generate(testData.count, 2);
  WriteFile(FILENAME, randomWriteData.data(), testData.count, testData.offset);

  EXPECT_TRUE(testFile->fileContentEquals(randomWriteData, testData.offset));
}

TEST_P(FuseWriteDataTest, RestOfFileIsUnchanged) {
  Data randomWriteData = DataFixture::generate(testData.count, 2);
  WriteFile(FILENAME, randomWriteData.data(), testData.count, testData.offset);

  EXPECT_TRUE(testFile->sizeUnchanged());
  EXPECT_TRUE(testFile->regionUnchanged(0, testData.offset));
  EXPECT_TRUE(testFile->regionUnchanged(testData.offset + testData.count, testData.additional_bytes_at_end_of_file));
}
