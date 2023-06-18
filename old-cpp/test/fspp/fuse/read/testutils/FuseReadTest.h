#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_READ_TESTUTILS_FUSEREADTEST_H_
#define MESSMER_FSPP_TEST_FUSE_READ_TESTUTILS_FUSEREADTEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseReadTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  struct ReadError {
    int error{};
    fspp::num_bytes_t read_bytes;
  };

  void ReadFile(const char *filename, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset);
  ReadError ReadFileReturnError(const char *filename, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset);

  ::testing::Action<fspp::num_bytes_t(int, void*, fspp::num_bytes_t, fspp::num_bytes_t)> ReturnSuccessfulRead =
    ::testing::Invoke([](int, void *, fspp::num_bytes_t count, fspp::num_bytes_t) {
      return count;
    });

  ::testing::Action<fspp::num_bytes_t(int, void*, fspp::num_bytes_t, fspp::num_bytes_t)> ReturnSuccessfulReadRegardingSize(fspp::num_bytes_t filesize) {
    return ::testing::Invoke([filesize](int, void *, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
      fspp::num_bytes_t ableToReadCount = std::min(count, filesize - offset);
      return ableToReadCount;
    });
  }

private:
  cpputils::unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
