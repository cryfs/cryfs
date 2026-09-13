#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cryfs-cli/CallAfterTimeout.h>
#include <atomic>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::chrono::milliseconds;
using boost::chrono::seconds;
using boost::chrono::steady_clock;
using boost::this_thread::sleep_for;
using namespace cryfs_cli;

// These tests used to check `called` after fixed sleeps, with 50 ms of margin between the timeout
// and the check. On a busy CI runner (the test binaries run in parallel there) either the timer
// thread or the test thread can be late by more than that, and the tests failed now and then on
// Linux and macOS. Now they only assert what holds however slow the machine is: the callback
// doesn't come before the timeout (its time is taken in the callback itself), it does come
// eventually, and a reset postpones it by a full timeout.
class CallAfterTimeoutTest : public ::testing::Test {
public:
    CallAfterTimeoutTest(): called(false), calledAt() {}

    unique_ref<CallAfterTimeout> callAfterTimeout(milliseconds timeout) {
        return make_unique_ref<CallAfterTimeout>(timeout, [this] {
            // Written before the atomic store to `called`, so it's visible to whoever saw `called`.
            calledAt = steady_clock::now();
            called = true;
        }, "test");
    }

    void waitUntilCalled() {
        const auto deadline = steady_clock::now() + seconds(30);
        while (!called) {
            ASSERT_LT(steady_clock::now(), deadline) << "The callback wasn't called within 30 seconds";
            sleep_for(milliseconds(5));
        }
    }

    std::atomic<bool> called;
    steady_clock::time_point calledAt;
};

TEST_F(CallAfterTimeoutTest, NoReset_1) {
    const auto start = steady_clock::now();
    auto obj = callAfterTimeout(milliseconds(100));
    waitUntilCalled();
    EXPECT_GE(calledAt - start, milliseconds(100));
}

TEST_F(CallAfterTimeoutTest, NoReset_2) {
    const auto start = steady_clock::now();
    auto obj = callAfterTimeout(milliseconds(500));
    waitUntilCalled();
    EXPECT_GE(calledAt - start, milliseconds(500));
}

TEST_F(CallAfterTimeoutTest, DoesntCallTwice) {
    auto obj = callAfterTimeout(milliseconds(50));
    waitUntilCalled();
    // Test that it isn't called again
    called = false;
    sleep_for(milliseconds(300));
    EXPECT_FALSE(called);
}

TEST_F(CallAfterTimeoutTest, OneReset) {
    auto obj = callAfterTimeout(milliseconds(1000));
    sleep_for(milliseconds(100));
    const auto resetAt = steady_clock::now();
    obj->resetTimer();
    // The timeout is ten times the sleep, so this only fails if the test thread got no CPU for
    // most of a second. Without it, the assertion below couldn't tell whether the reset worked.
    ASSERT_FALSE(called) << "The callback was called before the timer could be reset";
    waitUntilCalled();
    EXPECT_GE(calledAt - resetAt, milliseconds(1000));
}

TEST_F(CallAfterTimeoutTest, TwoResets) {
    auto obj = callAfterTimeout(milliseconds(1000));
    sleep_for(milliseconds(100));
    obj->resetTimer();
    sleep_for(milliseconds(100));
    const auto secondResetAt = steady_clock::now();
    obj->resetTimer();
    ASSERT_FALSE(called) << "The callback was called before the timer could be reset";
    waitUntilCalled();
    EXPECT_GE(calledAt - secondResetAt, milliseconds(1000));
}
