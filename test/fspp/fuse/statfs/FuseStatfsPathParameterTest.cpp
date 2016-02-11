#include "testutils/FuseStatfsTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Return;

class FuseStatfsPathParameterTest: public FuseStatfsTest {
};

TEST_F(FuseStatfsPathParameterTest, PathParameterIsCorrectRoot) {
  EXPECT_CALL(fsimpl, statfs(StrEq("/"), _)).Times(1).WillOnce(Return());
  Statfs("/");
}

TEST_F(FuseStatfsPathParameterTest, PathParameterIsCorrectSimpleFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(fsimpl, statfs(StrEq("/myfile"), _)).Times(1).WillOnce(Return());
  Statfs("/myfile");
}

TEST_F(FuseStatfsPathParameterTest, PathParameterIsCorrectSimpleDir) {
  ReturnIsDirOnLstat("/mydir");
  EXPECT_CALL(fsimpl, statfs(StrEq("/mydir"), _)).Times(1).WillOnce(Return());
  Statfs("/mydir");
}

TEST_F(FuseStatfsPathParameterTest, PathParameterIsCorrectNestedFile) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(fsimpl, statfs(StrEq("/mydir/mydir2/myfile"), _)).Times(1).WillOnce(Return());
  Statfs("/mydir/mydir2/myfile");
}

TEST_F(FuseStatfsPathParameterTest, PathParameterIsCorrectNestedDir) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsDirOnLstat("/mydir/mydir2/mydir3");
  EXPECT_CALL(fsimpl, statfs(StrEq("/mydir/mydir2/mydir3"), _)).Times(1).WillOnce(Return());
  Statfs("/mydir/mydir2/mydir3");
}
