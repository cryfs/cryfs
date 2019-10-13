#include "testutils/FuseLstatTest.h"

using ::testing::_;
using ::testing::Eq;
using ::testing::AtLeast;

class FuseLstatPathParameterTest: public FuseLstatTest {
};

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectRoot) {
  EXPECT_CALL(*fsimpl, lstat(Eq("/"), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsDir);
  LstatPath("/");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectSimpleFile) {
  EXPECT_CALL(*fsimpl, lstat(Eq("/myfile"), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsFile);
  LstatPath("/myfile");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectSimpleDir) {
  EXPECT_CALL(*fsimpl, lstat(Eq("/mydir"), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsDir);
  LstatPath("/mydir/");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectNestedFile) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(*fsimpl, lstat(Eq("/mydir/mydir2/myfile"), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsFile);
  LstatPath("/mydir/mydir2/myfile");
}

TEST_F(FuseLstatPathParameterTest, PathParameterIsCorrectNestedDir) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(*fsimpl, lstat(Eq("/mydir/mydir2/mydir3"), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsDir);
  LstatPath("/mydir/mydir2/mydir3/");
}
