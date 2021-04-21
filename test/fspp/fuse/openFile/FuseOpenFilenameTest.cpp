#include "testutils/FuseOpenTest.h"

using ::testing::Eq;
using ::testing::Return;

class FuseOpenFilenameTest: public FuseOpenTest {
public:
};

TEST_F(FuseOpenFilenameTest, OpenFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, openFile(Eq("/myfile"), testing::_))
    .Times(1).WillOnce(Return(0));

  OpenFile("/myfile", O_RDONLY);
}

TEST_F(FuseOpenFilenameTest, OpenFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, openFile(Eq("/mydir/myfile"), testing::_))
    .Times(1).WillOnce(Return(0));

  OpenFile("/mydir/myfile", O_RDONLY);
}

TEST_F(FuseOpenFilenameTest, OpenFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, openFile(Eq("/mydir/mydir2/myfile"), testing::_))
    .Times(1).WillOnce(Return(0));

  OpenFile("/mydir/mydir2/myfile", O_RDONLY);
}
