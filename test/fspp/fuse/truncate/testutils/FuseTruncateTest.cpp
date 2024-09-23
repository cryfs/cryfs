#include "FuseTruncateTest.h"
#include "fspp/fs_interface/Types.h"
#include <cerrno>
#include <gtest/gtest.h>
#include <unistd.h>

void FuseTruncateTest::TruncateFile(const char *filename, fspp::num_bytes_t size) {
  const int error = TruncateFileReturnError(filename, size);
  EXPECT_EQ(0, error);
}

int FuseTruncateTest::TruncateFileReturnError(const char *filename, fspp::num_bytes_t size) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  const int retval = ::truncate(realpath.string().c_str(), size.value());
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}
