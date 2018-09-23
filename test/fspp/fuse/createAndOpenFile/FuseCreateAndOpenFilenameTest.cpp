#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Return;

class FuseCreateAndOpenFilenameTest: public FuseCreateAndOpenTest {
public:
};

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFile) {
  ReturnDoesntExistOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(StrEq("/myfile"), _, _, _))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(StrEq("/mydir/myfile"), _, _, _))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/mydir/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnDoesntExistOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(StrEq("/mydir/mydir2/myfile"), _, _, _))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile("/mydir/mydir2/myfile", O_RDONLY);
}
