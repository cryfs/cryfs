#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::Eq;
using ::testing::Return;

class FuseCreateAndOpenFilenameTest: public FuseCreateAndOpenTest {
public:
};

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFile) {
  ReturnDoesntExistOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/mydir/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnDoesntExistOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/mydir2/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/mydir/mydir2/myfile", O_RDONLY);
}
