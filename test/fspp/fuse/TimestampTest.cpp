#include "../testutils/FuseTest.h"

#include <gmock/gmock.h>

using namespace fspp::fuse;

typedef FuseTest FuseTimestampTest;

// Single flag

TEST_F(FuseTimestampTest, whenCalledWithoutAnyAtimeFlag_thenHasRelatimeBehavior) {
    auto fs = TestFS({});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeFlag_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "noatime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeFlag_thenHasStrictatimeBehavior) {
    auto fs = TestFS({"-o", "strictatime"});
    EXPECT_EQ(fspp::strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeFlag_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeFlag_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeFlag_thenHasNoatimeBehavior) {
    // note: this behavior is correct because "noatime" is default and adding "nodiratime" doesn't change anything.
    auto fs = TestFS({"-o", "nodiratime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}



// Flag combinations

TEST_F(FuseTimestampTest, whenCalledWithAtimeAtimeFlag_withCsv_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime,atime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeAtimeFlag_withSeparateFlags_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime", "-o", "atime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeNoatimeFlag_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "atime,noatime"}),
        "Cannot have both, noatime and atime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeNoatimeFlag_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "atime", "-o", "noatime"}),
        "Cannot have both, noatime and atime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeRelatimeFlag_withCsv_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime,relatime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeRelatimeFlag_withSeparateFlags_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime", "-o", "relatime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeStrictatimeFlag_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "atime,strictatime"}),
        "Cannot have both, atime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeStrictatimeFlag_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "atime", "-o", "strictatime"}),
        "Cannot have both, atime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeNodiratimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime,nodiratime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithAtimeNodiratimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "atime", "-o", "nodiratime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeAtime_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime,atime"}),
        "Cannot have both, noatime and atime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeAtime_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime", "-o", "atime"}),
        "Cannot have both, noatime and atime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeNoatimeFlag_withCsv_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "noatime,noatime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeNoatimeFlag_withSeparateFlags_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "noatime", "-o", "noatime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeRelatime_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime,relatime"}),
        "Cannot have both, noatime and relatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeRelatime_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime", "-o", "relatime"}),
        "Cannot have both, noatime and relatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeStrictatime_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime,strictatime"}),
        "Cannot have both, noatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeStrictatime_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "noatime", "-o", "strictatime"}),
        "Cannot have both, noatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeNodiratimeFlag_withCsv_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "noatime,nodiratime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNoatimeNodiratimeFlag_withSeparateFlags_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "noatime", "-o", "nodiratime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeAtimeFlag_withCsv_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime,atime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeAtimeFlag_withSeparateFlags_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime", "-o", "atime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeNoatime_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "relatime,noatime"}),
        "Cannot have both, noatime and relatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeNoatime_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "relatime", "-o", "noatime"}),
        "Cannot have both, noatime and relatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeRelatimeFlag_withCsv_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime,relatime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeRelatimeFlag_withSeparateFlags_thenHasRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime", "-o", "relatime"});
    EXPECT_EQ(fspp::relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeStrictatime_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "relatime,strictatime"}),
        "Cannot have both, relatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeStrictatime_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "relatime", "-o", "strictatime"}),
        "Cannot have both, relatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeNodiratimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime,nodiratime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithRelatimeNodiratimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "relatime", "-o", "nodiratime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeAtimeFlag_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime,atime"}),
        "Cannot have both, atime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeAtimeFlag_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime", "-o", "atime"}),
        "Cannot have both, atime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeNoatimeFlag_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime,noatime"}),
        "Cannot have both, noatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeNoatimeFlag_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime", "-o", "noatime"}),
        "Cannot have both, noatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeRelatimeFlag_withCsv_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime,relatime"}),
        "Cannot have both, relatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeRelatimeFlag_withSeparateFlags_thenFails) {
    EXPECT_DEATH(
        TestFS({"-o", "strictatime", "-o", "relatime"}),
        "Cannot have both, relatime and strictatime flags set.");
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeStrictatimeFlag_withCsv_thenHasStrictatimeBehavior) {
    auto fs = TestFS({"-o", "strictatime,strictatime"});
    EXPECT_EQ(fspp::strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeStrictatimeFlag_withSeparateFlags_thenHasStrictatimeBehavior) {
    auto fs = TestFS({"-o", "strictatime", "-o", "strictatime"});
    EXPECT_EQ(fspp::strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeNodiratimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "strictatime,nodiratime"});
    EXPECT_EQ(fspp::nodiratime_strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithStrictatimeNodiratimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "strictatime", "-o", "nodiratime"});
    EXPECT_EQ(fspp::nodiratime_strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeAtimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime,atime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeAtimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime", "-o", "atime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeNoatimeFlag_withCsv_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime,noatime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeNoatimeFlag_withSeparateFlags_thenHasNoatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime", "-o", "noatime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeRelatimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime,relatime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeRelatimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime", "-o", "relatime"});
    EXPECT_EQ(fspp::nodiratime_relatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeStrictatimeFlag_withCsv_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime,strictatime"});
    EXPECT_EQ(fspp::nodiratime_strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeStrictatimeFlag_withSeparateFlags_thenHasNodiratimeRelatimeBehavior) {
    auto fs = TestFS({"-o", "nodiratime", "-o", "strictatime"});
    EXPECT_EQ(fspp::nodiratime_strictatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeNodiratimeFlag_withCsv_thenHasNoatimeBehavior) {
    // note: this behavior is correct because "noatime" is default and adding "nodiratime" doesn't change anything.
    auto fs = TestFS({"-o", "nodiratime,nodiratime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}

TEST_F(FuseTimestampTest, whenCalledWithNodiratimeNodiratimeFlag_withSeparateFlags_thenHasNoatimeBehavior) {
    // note: this behavior is correct because "noatime" is default and adding "nodiratime" doesn't change anything.
    auto fs = TestFS({"-o", "nodiratime", "-o", "nodiratime"});
    EXPECT_EQ(fspp::noatime().get(), context().timestampUpdateBehavior().get());
}
