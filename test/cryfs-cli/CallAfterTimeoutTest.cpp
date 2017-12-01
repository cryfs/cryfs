#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cryfs-cli/CallAfterTimeout.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::chrono::milliseconds;
using boost::chrono::minutes;
using boost::this_thread::sleep_for;
using namespace cryfs;

class CallAfterTimeoutTest : public ::testing::Test {
public:
    unique_ref<CallAfterTimeout> callAfterTimeout(milliseconds timeout) {
        return make_unique_ref<CallAfterTimeout>(timeout, [this] {called = true;});
    }

    bool called = false;
};

TEST_F(CallAfterTimeoutTest, NoReset_1) {
    auto obj = callAfterTimeout(milliseconds(100));
    sleep_for(milliseconds(50));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(100));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, NoReset_2) {
    auto obj = callAfterTimeout(milliseconds(200));
    sleep_for(milliseconds(150));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(100));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, DoesntCallTwice) {
    auto obj = callAfterTimeout(milliseconds(50));
    // Wait until it was called
    while(!called) {
        sleep_for(milliseconds(10));
    }
    EXPECT_TRUE(called);
    // Test that it isn't called again
    called = false;
    sleep_for(milliseconds(150));
    EXPECT_FALSE(called);
}

TEST_F(CallAfterTimeoutTest, OneReset) {
    auto obj = callAfterTimeout(milliseconds(200));
    sleep_for(milliseconds(125));
    obj->resetTimer();
    sleep_for(milliseconds(125));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(125));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, TwoResets) {
    auto obj = callAfterTimeout(milliseconds(100));
    sleep_for(milliseconds(50));
    obj->resetTimer();
    sleep_for(milliseconds(75));
    obj->resetTimer();
    sleep_for(milliseconds(75));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(75));
    EXPECT_TRUE(called);
}
