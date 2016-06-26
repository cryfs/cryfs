#include <gtest/gtest.h>

#include "../../../src/cryfs/filesystem/MyClientId.h"
#include <cpp-utils/tempfile/TempDir.h>

using cpputils::TempDir;
using cryfs::MyClientId;

class MyClientIdTest : public ::testing::Test {
public:
    TempDir stateDir;
    TempDir stateDir2;
};

TEST_F(MyClientIdTest, ValueIsConsistent) {
    uint32_t myClientId = MyClientId(stateDir.path()).loadOrGenerate();
    EXPECT_EQ(myClientId, MyClientId(stateDir.path()).loadOrGenerate());
}

TEST_F(MyClientIdTest, ValueIsRandomForNewClient) {
    uint32_t myClientId = MyClientId(stateDir.path()).loadOrGenerate();
    EXPECT_NE(myClientId, MyClientId(stateDir2.path()).loadOrGenerate());
}
