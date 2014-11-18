#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include <string>
#include <thread>
#include <csignal>
#include <fcntl.h>

#include "cryfs_lib/CryDevice.h"
#include "test/testutils/FuseThread.h"

#include "fspp/fuse/Fuse.h"
#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;
using std::string;
using std::unique_ptr;
using std::make_unique;
using std::vector;
using ::testing::Return;
using ::testing::_;
using ::testing::Invoke;
using ::testing::Throw;
using ::testing::NiceMock;
using ::testing::StrictMock;
using ::testing::AtMost;
using ::testing::Mock;
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
