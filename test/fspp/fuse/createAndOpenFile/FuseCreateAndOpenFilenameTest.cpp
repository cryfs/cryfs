#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::Eq;
using ::testing::Return;

class FuseCreateAndOpenFilenameTest: public FuseCreateAndOpenTest {
public:
  static constexpr int DESCRIPTOR = 0;
};

// After create(), libfuse asks for the new file's attributes through the handle create() just
// returned, so the answer comes from fstat() and the path is never looked up a second time.

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFile) {
  ReturnDoesntExistOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(DESCRIPTOR));
  ReturnIsFileOnFstat(DESCRIPTOR);

  CreateAndOpenFile("/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(DESCRIPTOR));
  ReturnIsFileOnFstat(DESCRIPTOR);

  CreateAndOpenFile("/mydir/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnDoesntExistOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/mydir2/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(DESCRIPTOR));
  ReturnIsFileOnFstat(DESCRIPTOR);

  CreateAndOpenFile("/mydir/mydir2/myfile", O_RDONLY);
}
