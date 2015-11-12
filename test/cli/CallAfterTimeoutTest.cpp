#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include "../../src/cli/CallAfterTimeout.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::chrono::milliseconds;
using boost::chrono::minutes;
using boost::chrono::duration_cast;
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
    auto obj = callAfterTimeout(milliseconds(50));
    sleep_for(milliseconds(40));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(20));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, NoReset_2) {
    auto obj = callAfterTimeout(milliseconds(100));
    sleep_for(milliseconds(90));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(20));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, DoesntCallTwice) {
    auto obj = callAfterTimeout(milliseconds(50));
    sleep_for(milliseconds(60));
    EXPECT_TRUE(called);
    called = false;
    sleep_for(milliseconds(60));
    EXPECT_FALSE(called);
}

TEST_F(CallAfterTimeoutTest, OneReset) {
    auto obj = callAfterTimeout(milliseconds(50));
    sleep_for(milliseconds(40));
    obj->resetTimer();
    sleep_for(milliseconds(40));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(20));
    EXPECT_TRUE(called);
}

TEST_F(CallAfterTimeoutTest, TwoResets) {
    auto obj = callAfterTimeout(milliseconds(50));
    sleep_for(milliseconds(20));
    obj->resetTimer();
    sleep_for(milliseconds(40));
    obj->resetTimer();
    sleep_for(milliseconds(40));
    EXPECT_FALSE(called);
    sleep_for(milliseconds(20));
    EXPECT_TRUE(called);
}
