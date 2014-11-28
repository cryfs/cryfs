#include "testutils/FuseUtimensTest.h"
#include "gtest/gtest.h"
#include "gmock/gmock.h"


using ::testing::_;
using ::testing::StrEq;
using ::testing::Return;

class FuseUtimensFilenameTest: public FuseUtimensTest {
};

TEST_F(FuseUtimensFilenameTest, UtimensFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(fsimpl, utimens(StrEq("/myfile"), _))
    .Times(1).WillOnce(Return());

  Utimens("/myfile", TIMEVALUES);
}

TEST_F(FuseUtimensFilenameTest, UtimensFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(fsimpl, utimens(StrEq("/mydir/myfile"), _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/myfile", TIMEVALUES);
}

TEST_F(FuseUtimensFilenameTest, UtimensFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(fsimpl, utimens(StrEq("/mydir/mydir2/myfile"), _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2/myfile", TIMEVALUES);
}

TEST_F(FuseUtimensFilenameTest, UtimensDir) {
  ReturnIsDirOnLstat("/mydir");
  EXPECT_CALL(fsimpl, utimens(StrEq("/mydir"), _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir", TIMEVALUES);
}

TEST_F(FuseUtimensFilenameTest, UtimensDirNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, utimens(StrEq("/mydir/mydir2"), _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2", TIMEVALUES);
}

TEST_F(FuseUtimensFilenameTest, UtimensDirNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsDirOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(fsimpl, utimens(StrEq("/mydir/mydir2/mydir3"), _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2/mydir3", TIMEVALUES);
}
