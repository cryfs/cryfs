#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::Eq;
using ::testing::Invoke;

class FuseCreateAndOpenFilenameTest: public FuseCreateAndOpenTest {
public:
};

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFile) {
  bool created = false;
  ReturnIsFileOnLstatIfFlagIsSet("/myfile", &created);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Invoke([&] () {
      ASSERT(!created, "called createAndOpenFile multiple times");
      created = true;
      return 0;
    }));

  CreateAndOpenFile("/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested) {
  bool created = false;
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstatIfFlagIsSet("/mydir/myfile", &created);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Invoke([&] () {
      ASSERT(!created, "called createAndOpenFile multiple times");
      created = true;
      return 0;
    }));

  CreateAndOpenFile("/mydir/myfile", O_RDONLY);
}

TEST_F(FuseCreateAndOpenFilenameTest, CreateAndOpenFileNested2) {
  bool created = false;
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstatIfFlagIsSet("/mydir/mydir2/myfile", &created);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq("/mydir/mydir2/myfile"), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Invoke([&] () {
      ASSERT(!created, "called createAndOpenFile multiple times");
      created = true;
      return 0;
    }));

  CreateAndOpenFile("/mydir/mydir2/myfile", O_RDONLY);
}
