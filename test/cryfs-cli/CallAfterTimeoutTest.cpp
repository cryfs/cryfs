#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cryfs-cli/CallAfterTimeout.h>
#include <atomic>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::chrono::milliseconds;
using boost::chrono::steady_clock;
using boost::chrono::duration_cast;
using boost::this_thread::sleep_for;
using namespace cryfs_cli;

class CallAfterTimeoutTest : public ::testing::Test {
public:
    CallAfterTimeoutTest(): _timerStartedAt(), _calledAt(), _called(false) {}

    // Creates a CallAfterTimeout object and remembers when its timer was started.
    // We take our timestamp before the constructor takes its own one, so the deadline the object
    // uses is guaranteed to be at or after _timerStartedAt + timeout. expectCalledAfter() relies
    // on that.
    unique_ref<CallAfterTimeout> callAfterTimeout(milliseconds timeout) {
        _timerStartedAt = steady_clock::now();
        return make_unique_ref<CallAfterTimeout>(timeout, [this] {_rememberCall();}, "test");
    }

    // Restarts the timer. Same reasoning as above: we remember the time before restarting it, so
    // the new deadline can only be later than what we remember, never earlier.
    void resetTimer(CallAfterTimeout* obj) {
        _timerStartedAt = steady_clock::now();
        obj->resetTimer();
    }

    // Waits for the callback and checks that it didn't happen earlier than 'timeout' after the
    // timer was last started.
    //
    // We deliberately don't test this by checking "not called yet" at some point in time before
    // the timeout elapses. Such a check needs a sleep shorter than the timeout, but a sleep can
    // only ever take longer than requested, never shorter, so on a loaded machine it can wake up
    // after the timeout already elapsed and then fail even though the class behaved correctly.
    // Looking at when the call actually happened tests the same property and can't be thrown off
    // by a slow machine.
    void expectCalledAfter(milliseconds timeout) {
        ASSERT_TRUE(_waitForCall(timeout + maxWaitForExpectedCall()))
            << "Callback wasn't called within " << (timeout + maxWaitForExpectedCall()).count() << "ms";
        const steady_clock::duration delay = _calledAt - _timerStartedAt;
        EXPECT_GE(delay, timeout)
            << "Callback was called " << duration_cast<milliseconds>(delay).count()
            << "ms after the timer was started, but it shouldn't be called before " << timeout.count() << "ms";
    }

    // Checks that the callback isn't called again after it was already called once.
    void expectNotCalledAgain() {
        _called = false;
        EXPECT_FALSE(_waitForCall(waitForUnexpectedCall())) << "Callback was called a second time";
    }

private:
    // How much longer than the timeout we wait for a callback we expect to happen. This is only
    // reached when the callback doesn't happen at all, so it can be generous.
    static milliseconds maxWaitForExpectedCall() { return milliseconds(10000); }

    // How long we wait to make sure a callback we don't expect really doesn't happen.
    static milliseconds waitForUnexpectedCall() { return milliseconds(150); }

    void _rememberCall() {
        _calledAt = steady_clock::now();
        // Set this last. Writing it publishes _calledAt to the thread reading _called.
        _called = true;
    }

    // Waits until the callback happened or maxWait elapsed, and returns whether it happened.
    bool _waitForCall(milliseconds maxWait) {
        const steady_clock::time_point deadline = steady_clock::now() + maxWait;
        while (!_called && steady_clock::now() < deadline) {
            sleep_for(milliseconds(1));
        }
        return _called;
    }

    steady_clock::time_point _timerStartedAt;
    steady_clock::time_point _calledAt;
    std::atomic<bool> _called;
};

TEST_F(CallAfterTimeoutTest, NoReset_1) {
    auto obj = callAfterTimeout(milliseconds(100));
    expectCalledAfter(milliseconds(100));
}

TEST_F(CallAfterTimeoutTest, NoReset_2) {
    auto obj = callAfterTimeout(milliseconds(200));
    expectCalledAfter(milliseconds(200));
}

TEST_F(CallAfterTimeoutTest, DoesntCallTwice) {
    auto obj = callAfterTimeout(milliseconds(50));
    expectCalledAfter(milliseconds(50));
    expectNotCalledAgain();
}

TEST_F(CallAfterTimeoutTest, OneReset) {
    auto obj = callAfterTimeout(milliseconds(200));
    sleep_for(milliseconds(125));
    // Without the reset, the callback would happen 75ms from here. expectCalledAfter() checks it
    // doesn't happen before 200ms from here, i.e. that the reset actually restarted the timer.
    resetTimer(obj.get());
    expectCalledAfter(milliseconds(200));
}

TEST_F(CallAfterTimeoutTest, TwoResets) {
    auto obj = callAfterTimeout(milliseconds(200));
    sleep_for(milliseconds(100));
    resetTimer(obj.get());
    sleep_for(milliseconds(125));
    resetTimer(obj.get());
    expectCalledAfter(milliseconds(200));
}
