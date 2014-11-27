#include "testutils/FuseReadTest.h"

#include "fspp/impl/FuseErrnoException.h"

#include <tuple>
#include <cstdlib>

using ::testing::_;
using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Combine;
using ::testing::Eq;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::Action;

using std::tuple;
using std::get;
using std::min;

using namespace fspp;

// We can't test the count or size parameter directly, because fuse doesn't pass them 1:1.
// It usually asks to read bigger blocks (probably does some caching).
// But we can test that the data returned from the ::read syscall is the correct data region.

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

// The testcase creates random data in memory, offers a mock read() implementation to read from this
// memory region and check methods to check for data equality of a region.
class FuseReadReturnedDataTest: public FuseReadTest, public WithParamInterface<tuple<size_t, off_t, size_t>> {
public:
  char *fileData;
  TestData testData;

  void SetUp() override {
    testData = GetParam();
    setupFileData();

    ReturnIsFileOnLstatWithSize(FILENAME, testData.fileSize());
    OnOpenReturnFileDescriptor(FILENAME, 0);
    EXPECT_CALL(fsimpl, read(0, _, _, _))
      .WillRepeatedly(ReadFromFile);
  }

  void TearDown() override {
    delete[] fileData;
  }

  // Return true, iff the given data is equal to the data of the file at the given offset.
  bool fileContentCorrect(char *content, size_t count, off_t offset) {
    return 0 == memcmp(content, fileData + offset, count);
  }

  // This read() mock implementation reads from the stored random data.
  Action<int(int, void*, size_t, off_t)> ReadFromFile = Invoke([this](int, void *buf, size_t count, off_t offset) {
    size_t realCount = min(count, testData.fileSize() - offset);
    memcpy(buf, fileData+offset, realCount);
    return realCount;
  });
private:
  void setupFileData() {
    fileData = new char[testData.fileSize()];
    fillFileWithRandomData();
  }
  void fillFileWithRandomData() {
    long long int val = 1;
    for(unsigned int i=0; i<testData.fileSize()/sizeof(long long int); ++i) {
      //MMIX linear congruential generator
      val *= 6364136223846793005L;
      val += 1442695040888963407;
      reinterpret_cast<long long int*>(fileData)[i] = val;
    }
  }
};
INSTANTIATE_TEST_CASE_P(FuseReadReturnedDataTest, FuseReadReturnedDataTest, Combine(Values(0,1,10,1000,1024, 10*1024*1024), Values(0, 1, 10, 1024, 10*1024*1024), Values(0, 1, 10, 1024, 10*1024*1024)));


TEST_P(FuseReadReturnedDataTest, ReturnedDataRangeIsCorrect) {
  char *buf = new char[testData.count];
  ReadFile(FILENAME, buf, testData.count, testData.offset);
  EXPECT_TRUE(fileContentCorrect(buf, testData.count, testData.offset));
  delete[] buf;
}
