#include "fspp/fs_interface/FuseErrnoException.h"
#include "fspp/fs_interface/Types.h"
#include "testutils/FuseTruncateTest.h"
#include "gmock/gmock.h"
#include "gtest/gtest.h"
#include <cerrno>
#include <gtest/gtest.h>

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseTruncateErrorTest: public FuseTruncateTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseTruncateErrorTest, FuseTruncateErrorTest, Values(EACCES, EFAULT, EFBIG, EINTR, EINVAL, EIO, EISDIR, ELOOP, ENAMETOOLONG, ENOENT, ENOTDIR, EPERM, EROFS, ETXTBSY));

TEST_P(FuseTruncateErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, truncate(Eq(FILENAME), testing::_))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  const int error = TruncateFileReturnError(FILENAME, fspp::num_bytes_t(0));
  EXPECT_EQ(GetParam(), error);
}
