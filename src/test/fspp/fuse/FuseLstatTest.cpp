#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;

using ::testing::_;
using ::testing::StrEq;
using ::testing::Action;
using ::testing::Invoke;

using std::vector;
using std::string;

class FuseLstatTest: public FuseTest {
public:
  struct stat LstatPath(const string &path) {
    auto fs = TestFS();

    auto realpath = fs->mountDir() / path;
    struct stat stat;
    ::lstat(realpath.c_str(), &stat);

    return stat;
  }
};


TEST_F(FuseLstatTest, PathParameterIsCorrectRoot) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/"), _)).Times(1);
  LstatPath("/");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectSimpleFile) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/myfile"), _)).Times(1);
  LstatPath("/myfile");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectSimpleDir) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir"), _)).Times(1);
  LstatPath("/mydir/");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectNestedFile) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/myfile"), _)).Times(1);
  LstatPath("/mydir/mydir2/myfile");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectNestedDir) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/mydir3"), _)).Times(1);
  LstatPath("/mydir/mydir2/mydir3/");
}
