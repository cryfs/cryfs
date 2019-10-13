#include "testutils/FuseUnlinkTest.h"

using ::testing::Eq;

class FuseUnlinkFilenameTest: public FuseUnlinkTest {
};

TEST_F(FuseUnlinkFilenameTest, Unlink) {
  ReturnIsFileOnLstat("/mydir");
  EXPECT_CALL(*fsimpl, unlink(Eq("/mydir")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Unlink("/mydir");
}

TEST_F(FuseUnlinkFilenameTest, UnlinkNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/mysubdir");
  EXPECT_CALL(*fsimpl, unlink(Eq("/mydir/mysubdir")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Unlink("/mydir/mysubdir");
}

TEST_F(FuseUnlinkFilenameTest, UnlinkNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(*fsimpl, unlink(Eq("/mydir/mydir2/mydir3")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Unlink("/mydir/mydir2/mydir3");
}
