#include "testutils/FuseRenameTest.h"

#include <cerrno>

using ::testing::Eq;
using ::testing::Return;
using ::testing::_;

// libfuse 3 hands the renameat2() flags to the filesystem and expects it to honour them.
// CryFS cannot: RENAME_EXCHANGE needs an atomic swap and RENAME_NOREPLACE needs the
// existence check to happen inside the rename. Ignoring the flags is actively dangerous -
// a plain rename in response to RENAME_EXCHANGE reports success while destroying one of the
// two files. So we reject them, which is what the kernel did for us under libfuse 2.

#if defined(__linux__)

// From <linux/fs.h>; defined here so the test does not depend on that header being available.
#ifndef RENAME_NOREPLACE
#define RENAME_NOREPLACE (1 << 0)
#endif
#ifndef RENAME_EXCHANGE
#define RENAME_EXCHANGE (1 << 1)
#endif
#ifndef RENAME_WHITEOUT
#define RENAME_WHITEOUT (1 << 2)
#endif

class FuseRenameFlagsTest: public FuseRenameTest {
public:
  void ReturnTwoExistingFiles() {
    ReturnIsFileOnLstat(FILENAME1);
    ReturnIsFileOnLstat(FILENAME2);
  }
};

TEST_F(FuseRenameFlagsTest, RenameExchange_IsRejectedAndDoesNotReachTheFilesystem) {
  // Without this, the rename would go through as a plain rename: it would report success
  // while overwriting (and deleting) one of the two files.
  ReturnTwoExistingFiles();
  EXPECT_CALL(*fsimpl, rename(_, _)).Times(0);

  const int error = Renameat2ReturnError(FILENAME1, FILENAME2, RENAME_EXCHANGE);
  EXPECT_EQ(EINVAL, error);
}

TEST_F(FuseRenameFlagsTest, RenameNoreplace_IsRejectedAndDoesNotReachTheFilesystem) {
  ReturnIsFileOnLstat(FILENAME1);
  ReturnDoesntExistOnLstat(FILENAME2);
  EXPECT_CALL(*fsimpl, rename(_, _)).Times(0);

  const int error = Renameat2ReturnError(FILENAME1, FILENAME2, RENAME_NOREPLACE);
  EXPECT_EQ(EINVAL, error);
}

TEST_F(FuseRenameFlagsTest, RenameWhiteout_IsRejected) {
  ReturnIsFileOnLstat(FILENAME1);
  ReturnDoesntExistOnLstat(FILENAME2);
  EXPECT_CALL(*fsimpl, rename(_, _)).Times(0);

  const int error = Renameat2ReturnError(FILENAME1, FILENAME2, RENAME_WHITEOUT);
  EXPECT_EQ(EINVAL, error);
}

TEST_F(FuseRenameFlagsTest, WithoutFlags_TheRenameStillGoesThrough) {
  // Regression guard: rejecting flags must not break the ordinary rename path.
  ReturnIsFileOnLstat(FILENAME1);
  ReturnDoesntExistOnLstat(FILENAME2);
  EXPECT_CALL(*fsimpl, rename(Eq(FILENAME1), Eq(FILENAME2))).Times(1).WillOnce(Return());

  const int error = Renameat2ReturnError(FILENAME1, FILENAME2, 0);
  EXPECT_EQ(0, error);
}

#endif
