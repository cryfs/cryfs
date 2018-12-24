#include "testutils/FuseAccessTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Return;

class FuseAccessFilenameTest: public FuseAccessTest {
};

TEST_F(FuseAccessFilenameTest, AccessFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, access(StrEq("/myfile"), _))
    .Times(1).WillOnce(Return());

  AccessFile("/myfile", 0);
}

TEST_F(FuseAccessFilenameTest, AccessFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, access(StrEq("/mydir/myfile"), _))
    .Times(1).WillOnce(Return());

  AccessFile("/mydir/myfile", 0);
}

TEST_F(FuseAccessFilenameTest, AccessFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, access(StrEq("/mydir/mydir2/myfile"), _))
    .Times(1).WillOnce(Return());

  AccessFile("/mydir/mydir2/myfile", 0);
}
