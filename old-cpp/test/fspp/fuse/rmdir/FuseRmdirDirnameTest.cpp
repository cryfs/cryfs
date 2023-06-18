#include "testutils/FuseRmdirTest.h"

using ::testing::Eq;

class FuseRmdirDirnameTest: public FuseRmdirTest {
};

TEST_F(FuseRmdirDirnameTest, Rmdir) {
  ReturnIsDirOnLstat("/mydir");
  EXPECT_CALL(*fsimpl, rmdir(Eq("/mydir")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Rmdir("/mydir");
}

TEST_F(FuseRmdirDirnameTest, RmdirNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mysubdir");
  EXPECT_CALL(*fsimpl, rmdir(Eq("/mydir/mysubdir")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Rmdir("/mydir/mysubdir");
}

TEST_F(FuseRmdirDirnameTest, RmdirNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsDirOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(*fsimpl, rmdir(Eq("/mydir/mydir2/mydir3")))
    // After rmdir was called, lstat should return that it doesn't exist anymore
    // This is needed to make the ::rmdir() syscall pass.
    .Times(1).WillOnce(FromNowOnReturnDoesntExistOnLstat());

  Rmdir("/mydir/mydir2/mydir3");
}
