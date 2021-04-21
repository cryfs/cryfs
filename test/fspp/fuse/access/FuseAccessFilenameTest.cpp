#include "testutils/FuseAccessTest.h"

using ::testing::Eq;
using ::testing::Return;

class FuseAccessFilenameTest: public FuseAccessTest {
};

TEST_F(FuseAccessFilenameTest, AccessFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, access(Eq("/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  AccessFile("/myfile", 0);
}

TEST_F(FuseAccessFilenameTest, AccessFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, access(Eq("/mydir/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  AccessFile("/mydir/myfile", 0);
}

TEST_F(FuseAccessFilenameTest, AccessFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, access(Eq("/mydir/mydir2/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  AccessFile("/mydir/mydir2/myfile", 0);
}
