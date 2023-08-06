#include "testutils/FuseCreateAndOpenTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Throw;
using ::testing::Eq;
using ::testing::Invoke;

using namespace fspp::fuse;

class FuseCreateAndOpenErrorTest: public FuseCreateAndOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseCreateAndOpenErrorTest, FuseCreateAndOpenErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EOVERFLOW, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOPNOTSUPP, EPERM, EROFS, ETXTBSY, EWOULDBLOCK, EBADF, ENOTDIR));

TEST_F(FuseCreateAndOpenErrorTest, ReturnNoError) {
  bool created = false;
  ReturnIsFileOnLstatIfFlagIsSet(FILENAME, &created);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), testing::_, testing::_, testing::_)).Times(1).WillOnce(Invoke([&] () {
    ASSERT(!created, "called created multiple times");
    created = true;
    return 1;
  }));

  const int error = CreateAndOpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(0, error);
}

TEST_P(FuseCreateAndOpenErrorTest, ReturnError) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), testing::_, testing::_, testing::_)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  const int error = CreateAndOpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(GetParam(), error);
}
