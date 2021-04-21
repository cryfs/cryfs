#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/data/Data.h>
#include "../../testutils/InMemoryFile.h"
#include "testutils/FuseReadTest.h"
#include <cpp-utils/pointer/unique_ref.h>

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
// It usually asks to read bigger blocks (probably does some caching).
// But we can test that the data returned from the ::read syscall is the correct data region.

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

// The testcase creates random data in memory, offers a mock read() implementation to read from this
// memory region and check methods to check for data equality of a region.
class FuseReadReturnedDataTest: public FuseReadTest, public WithParamInterface<tuple<fspp::num_bytes_t, fspp::num_bytes_t, fspp::num_bytes_t>> {
public:
  std::unique_ptr<InMemoryFile> testFile;
  TestData testData;

  FuseReadReturnedDataTest()
          : testFile(nullptr),
            testData(GetParam()) {
    testFile = std::make_unique<InMemoryFile>(DataFixture::generate(testData.fileSize().value()));
    ReturnIsFileOnLstatWithSize(FILENAME, testData.fileSize());
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(*fsimpl, read(0, testing::_, testing::_, testing::_))
      .WillRepeatedly(ReadFromFile);
  }

  // This read() mock implementation reads from the stored virtual file (testFile).
  Action<fspp::num_bytes_t(int, void*, fspp::num_bytes_t, fspp::num_bytes_t)> ReadFromFile = Invoke([this](int, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
    return testFile->read(buf, count, offset);
  });
};
INSTANTIATE_TEST_SUITE_P(FuseReadReturnedDataTest, FuseReadReturnedDataTest, Combine(
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1000), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024)),
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024)),
        Values(fspp::num_bytes_t(0), fspp::num_bytes_t(1), fspp::num_bytes_t(10), fspp::num_bytes_t(1024), fspp::num_bytes_t(10*1024*1024))
        ));


TEST_P(FuseReadReturnedDataTest, ReturnedDataRangeIsCorrect) {
  Data buf(testData.count.value());
  ReadFile(FILENAME, buf.data(), testData.count, testData.offset);
  EXPECT_TRUE(testFile->fileContentEquals(buf, testData.offset));
}
