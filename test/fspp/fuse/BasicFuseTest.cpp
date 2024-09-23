#include "../testutils/FuseTest.h"
#include <gtest/gtest.h>

using namespace fspp::fuse;
using namespace fspp::fuse;


typedef FuseTest BasicFuseTest;

//This test case simply checks whether a filesystem can be setup and teardown without crashing.
TEST_F(BasicFuseTest, setupAndTearDown) {
  auto fs = TestFS();
}
