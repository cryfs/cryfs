#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;

using ::testing::_;
using ::testing::StrEq;

typedef FuseTest BasicFuseTest;

TEST_F(BasicFuseTest, setupAndTearDown) {
  //This test case simply checks whether a filesystem can be setup and teardown without crashing.
  auto fs = TestFS();
}
