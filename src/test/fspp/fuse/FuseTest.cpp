#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/fuse/Fuse.h"
#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;

using ::testing::_;
using ::testing::StrEq;

TEST_F(FuseTest, setupAndTearDown) {
  //This test case simply checks whether a filesystem can be setup and teardown without crashing.
  auto fs = TestFS();
}

TEST_F(FuseTest, openFile) {
  const char *filename = "/myfile";
  EXPECT_CALL(fsimpl, lstat(StrEq(filename), _)).WillOnce(ReturnIsFileStat);
  EXPECT_CALL(fsimpl, openFile(StrEq(filename), OpenFlagsEq(O_RDWR)))
    .Times(1);

  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  ::open(realpath.c_str(), O_RDWR);
}
