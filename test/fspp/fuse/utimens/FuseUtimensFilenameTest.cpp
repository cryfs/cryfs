#include "testutils/FuseUtimensTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Return;

class FuseUtimensFilenameTest: public FuseUtimensTest {
};

TEST_F(FuseUtimensFilenameTest, UtimensFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/myfile"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/myfile", TIMEVALUE, TIMEVALUE);
}

TEST_F(FuseUtimensFilenameTest, UtimensFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/mydir/myfile"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/myfile", TIMEVALUE, TIMEVALUE);
}

TEST_F(FuseUtimensFilenameTest, UtimensFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/mydir/mydir2/myfile"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2/myfile", TIMEVALUE, TIMEVALUE);
}

TEST_F(FuseUtimensFilenameTest, UtimensDir) {
  ReturnIsDirOnLstat("/mydir");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/mydir"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir", TIMEVALUE, TIMEVALUE);
}

TEST_F(FuseUtimensFilenameTest, UtimensDirNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/mydir/mydir2"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2", TIMEVALUE, TIMEVALUE);
}

TEST_F(FuseUtimensFilenameTest, UtimensDirNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsDirOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(*fsimpl, utimens(StrEq("/mydir/mydir2/mydir3"), _, _))
    .Times(1).WillOnce(Return());

  Utimens("/mydir/mydir2/mydir3", TIMEVALUE, TIMEVALUE);
}
