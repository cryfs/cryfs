#include <cpp-utils/data/DataFixture.h>
#include "testutils/FuseWriteTest.h"
#include "../../testutils/InMemoryFile.h"

#include "fspp/fs_interface/FuseErrnoException.h"

#include <tuple>
#include <cstdlib>

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
  TestData(const tuple<fspp::num_bytes_t, fspp::num_bytes_t, fspp::num_bytes_t> &data): count(get<0>(data)), offset(get<1>(data)), additional_bytes_at_end_of_file(get<2>(data)) {}
  fspp::num_bytes_t count;
  fspp::num_bytes_t offset;
  //How many more bytes does the file have after the read block?
  fspp::num_bytes_t additional_bytes_at_end_of_file;
  fspp::num_bytes_t fileSize() {
    return count + offset + additional_bytes_at_end_of_file;
  }
};

// The testcase creates random data in memory, offers a mock write() implementation to write to this
// memory region and check methods to check for data equality of a region.
class FuseWriteDataTest: public FuseWriteTest, public WithParamInterface<tuple<fspp::num_bytes_t, fspp::num_bytes_t, fspp::num_bytes_t>> {
public:
  std::unique_ptr<WriteableInMemoryFile> testFile;
  TestData testData;

  FuseWriteDataTest()
          : testFile(nullptr),
            testData(GetParam()) {
    testFile = std::make_unique<WriteableInMemoryFile>(DataFixture::generate(testData.fileSize().value(), 1));
    ReturnIsFileOnLstatWithSize(FILENAME, testData.fileSize());
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(*fsimpl, write(0, testing::_, testing::_, testing::_))
      .WillRepeatedly(WriteToFile);
  }

  // This write() mock implementation writes to the stored virtual file.
  Action<void(int, const void*, fspp::num_bytes_t, fspp::num_bytes_t)> WriteToFile = Invoke([this](int, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
    testFile->write(buf, count, offset);
  });
};
INSTANTIATE_TEST_SUITE_P(FuseWriteDataTest, FuseWriteDataTest, Combine(
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1000), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024)),
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024)),
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024))
        ));


TEST_P(FuseWriteDataTest, DataWasCorrectlyWritten) {
  Data randomWriteData = DataFixture::generate(testData.count.value(), 2);
  WriteFile(FILENAME, randomWriteData.data(), testData.count, testData.offset);

  EXPECT_TRUE(testFile->fileContentEquals(randomWriteData, testData.offset));
}

TEST_P(FuseWriteDataTest, RestOfFileIsUnchanged) {
  Data randomWriteData = DataFixture::generate(testData.count.value(), 2);
  WriteFile(FILENAME, randomWriteData.data(), testData.count, testData.offset);

  EXPECT_TRUE(testFile->sizeUnchanged());
  EXPECT_TRUE(testFile->regionUnchanged(fspp::num_bytes_t(0), testData.offset));
  EXPECT_TRUE(testFile->regionUnchanged(testData.offset + testData.count, testData.additional_bytes_at_end_of_file));
}
