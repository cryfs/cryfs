#include "testutils/FuseLstatTest.h"

using ::testing::_;
using ::testing::StrEq;

class FuseLstatPathParameterTest: public FuseLstatTest {
};

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectRoot) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectSimpleFile) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/myfile"), _)).Times(1).WillOnce(ReturnIsFile);
  LstatPath("/myfile");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectSimpleDir) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/mydir/");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectNestedFile) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/myfile"), _)).Times(1).WillOnce(ReturnIsFile);
  LstatPath("/mydir/mydir2/myfile");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectNestedDir) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/mydir3"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/mydir/mydir2/mydir3/");
}
