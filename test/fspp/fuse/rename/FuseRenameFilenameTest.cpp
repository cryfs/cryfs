#include "testutils/FuseRenameTest.h"

using ::testing::StrEq;
using ::testing::Return;

class FuseRenameFilenameTest: public FuseRenameTest {
};

TEST_F(FuseRenameFilenameTest, RenameFileRootToRoot) {
  ReturnIsFileOnLstat("/myfile");
  ReturnDoesntExistOnLstat("/myrenamedfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/myfile"), StrEq("/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/myfile", "/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameFileRootToNested) {
  ReturnIsFileOnLstat("/myfile");
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/mydir/myrenamedfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/myfile"), StrEq("/mydir/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/myfile", "/mydir/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameFileNestedToRoot) {
  ReturnDoesntExistOnLstat("/myrenamedfile");
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir/myfile"), StrEq("/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/mydir/myfile", "/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameFileNestedToNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  ReturnDoesntExistOnLstat("/mydir/myrenamedfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir/myfile"), StrEq("/mydir/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/mydir/myfile", "/mydir/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameFileNestedToNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  ReturnDoesntExistOnLstat("/mydir/mydir2/myrenamedfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir/mydir2/myfile"), StrEq("/mydir/mydir2/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/mydir/mydir2/myfile", "/mydir/mydir2/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameFileNestedToNested_DifferentFolder) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir2");
  ReturnIsFileOnLstat("/mydir/myfile");
  ReturnDoesntExistOnLstat("/mydir2/myrenamedfile");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir/myfile"), StrEq("/mydir2/myrenamedfile")))
    .Times(1).WillOnce(Return());

  Rename("/mydir/myfile", "/mydir2/myrenamedfile");
}

TEST_F(FuseRenameFilenameTest, RenameDirRootToRoot) {
  ReturnIsDirOnLstat("/mydir");
  ReturnDoesntExistOnLstat("/myrenameddir");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir"), StrEq("/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/mydir", "/myrenameddir");
}

TEST_F(FuseRenameFilenameTest, RenameDirRootToNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/myrootdir");
  ReturnDoesntExistOnLstat("/myrootdir/myrenameddir");
  EXPECT_CALL(fsimpl, rename(StrEq("/mydir"), StrEq("/myrootdir/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/mydir", "/myrootdir/myrenameddir");
}

TEST_F(FuseRenameFilenameTest, RenameDirNestedToRoot) {
  ReturnDoesntExistOnLstat("/myrenameddir");
  ReturnIsDirOnLstat("/myrootdir");
  ReturnIsDirOnLstat("/myrootdir/mydir");
  EXPECT_CALL(fsimpl, rename(StrEq("/myrootdir/mydir"), StrEq("/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/myrootdir/mydir", "/myrenameddir");
}

TEST_F(FuseRenameFilenameTest, RenameDirNestedToNested) {
  ReturnIsDirOnLstat("/myrootdir");
  ReturnIsDirOnLstat("/myrootdir/mydir");
  ReturnDoesntExistOnLstat("/myrootdir/myrenameddir");
  EXPECT_CALL(fsimpl, rename(StrEq("/myrootdir/mydir"), StrEq("/myrootdir/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/myrootdir/mydir", "/myrootdir/myrenameddir");
}

TEST_F(FuseRenameFilenameTest, RenameDirNestedToNested2) {
  ReturnIsDirOnLstat("/myrootdir");
  ReturnIsDirOnLstat("/myrootdir/myrootdir2");
  ReturnIsDirOnLstat("/myrootdir/myrootdir2/mydir");
  ReturnDoesntExistOnLstat("/myrootdir/myrootdir2/myrenameddir");
  EXPECT_CALL(fsimpl, rename(StrEq("/myrootdir/myrootdir2/mydir"), StrEq("/myrootdir/myrootdir2/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/myrootdir/myrootdir2/mydir", "/myrootdir/myrootdir2/myrenameddir");
}

TEST_F(FuseRenameFilenameTest, RenameDirNestedToNested_DifferentFolder) {
  ReturnIsDirOnLstat("/myrootdir");
  ReturnIsDirOnLstat("/myrootdir2");
  ReturnIsDirOnLstat("/myrootdir/mydir");
  ReturnDoesntExistOnLstat("/myrootdir2/myrenameddir");
  EXPECT_CALL(fsimpl, rename(StrEq("/myrootdir/mydir"), StrEq("/myrootdir2/myrenameddir")))
    .Times(1).WillOnce(Return());

  Rename("/myrootdir/mydir", "/myrootdir2/myrenameddir");
}
