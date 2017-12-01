#include "testutils/FuseMkdirTest.h"

using ::testing::_;
using ::testing::StrEq;

class FuseMkdirDirnameTest: public FuseMkdirTest {
};

TEST_F(FuseMkdirDirnameTest, Mkdir) {
  ReturnDoesntExistOnLstat("/mydir");
  EXPECT_CALL(fsimpl, mkdir(StrEq("/mydir"), _, _, _))
  // After mkdir was called, lstat should return that it is a dir.
  // This is needed to make the ::mkdir() syscall pass.
  .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  Mkdir("/mydir", 0);
}

TEST_F(FuseMkdirDirnameTest, MkdirNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/mydir/mysubdir");
  EXPECT_CALL(fsimpl, mkdir(StrEq("/mydir/mysubdir"), _, _, _))
  // After mkdir was called, lstat should return that it is a dir.
  // This is needed to make the ::mkdir() syscall pass.
  .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  Mkdir("/mydir/mysubdir", 0);
}

TEST_F(FuseMkdirDirnameTest, MkdirNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnDoesntExistOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(fsimpl, mkdir(StrEq("/mydir/mydir2/mydir3"), _, _, _))
  // After mkdir was called, lstat should return that it is a dir.
  // This is needed to make the ::mkdir() syscall pass.
  .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  Mkdir("/mydir/mydir2/mydir3", 0);
}
