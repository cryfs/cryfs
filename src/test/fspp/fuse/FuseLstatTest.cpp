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
  const char *FILENAME = "/myfile";
  const mode_t MODE1 = S_IFREG | S_IRUSR | S_IWGRP | S_IXOTH;
  const mode_t MODE2 = S_IFDIR | S_IWUSR | S_IXGRP | S_IROTH;

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

TEST_F(FuseLstatTest, ReturnedModeIsCorrect1) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).WillRepeatedly(Invoke([this](const char*, struct ::stat *stat) {
    stat->st_mode = MODE1;
  }));

  struct ::stat result = LstatPath(FILENAME);
  EXPECT_EQ(MODE1, result.st_mode);
}

TEST_F(FuseLstatTest, ReturnedModeIsCorrect2) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).WillRepeatedly(Invoke([this](const char*, struct ::stat *stat) {
    stat->st_mode = MODE2;
  }));

  struct ::stat result = LstatPath(FILENAME);
  EXPECT_EQ(MODE2, result.st_mode);
}
