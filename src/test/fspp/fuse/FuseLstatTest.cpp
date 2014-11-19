#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;

using ::testing::_;
using ::testing::StrEq;

typedef FuseTest FuseLstatTest;

TEST_F(FuseLstatTest, lstat) {
  const char *filename = "/myfile";
  EXPECT_CALL(fsimpl, lstat(StrEq(filename), _)).WillOnce(ReturnIsFileStat);

  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  struct stat stat;
  ::lstat(realpath.c_str(), &stat);

  EXPECT_TRUE(S_ISREG(stat.st_mode));
}
