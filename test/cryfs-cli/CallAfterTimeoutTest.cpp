#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cryfs-cli/CallAfterTimeout.h>
#include <atomic>
#include <functional>

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

    // Restarts the timer and remembers when. Same reasoning as above: we remember the time before
    // restarting it, so the new deadline can only be later than what we remember, never earlier.
    //
    // Returns false if the old timer had already expired and the callback already happened before
    // we got here. A sleep can only ever take longer than requested, never shorter, so on a loaded
    // machine a sleep that is meant to end before the timeout can end after it. There is nothing to
    // measure then and it says nothing about CallAfterTimeout, so the tests retry instead of
    // failing, see runUntilResetWasInTime().
    //
    // Checking _called here is reliable and not a race: CallAfterTimeout runs the callback while
    // holding the same mutex that resetTimer() takes, so once resetTimer() returned, the callback
    // either already happened or cannot happen before the new deadline anymore.
    bool resetTimer(CallAfterTimeout* obj) {
        const steady_clock::time_point resetAt = steady_clock::now();
        obj->resetTimer();
        if (_called) {
            return false;
        }
        _timerStartedAt = resetAt;
        return true;
    }

    // Runs 'scenario' until it managed to reset the timer before it expired. The scenario returns
    // what resetTimer() returned, i.e. false if a sleep overshot the timeout. Retrying lets the
    // checks in expectCalledAfter() stay strict without making the test flaky on a busy machine.
    void runUntilResetWasInTime(const std::function<bool()>& scenario) {
        for (int attempt = 0; attempt < maxAttempts(); ++attempt) {
            _called = false;
            if (scenario()) {
                return;
            }
        }
        FAIL() << "The timer expired before the test managed to reset it, in all " << maxAttempts()
               << " attempts. Either this machine is too busy to run timing tests, or the timer "
                  "expires earlier than it should.";
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

    // How often runUntilResetWasInTime() retries a scenario whose sleep overshot the timeout.
    static int maxAttempts() { return 5; }

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
    runUntilResetWasInTime([this] {
        auto obj = callAfterTimeout(milliseconds(200));
        sleep_for(milliseconds(125));
        // Without the reset, the callback would happen 75ms from here. expectCalledAfter() checks
        // it doesn't happen before 200ms from here, i.e. that the reset restarted the timer.
        if (!resetTimer(obj.get())) {
            return false;
        }
        expectCalledAfter(milliseconds(200));
        return true;
    });
}

TEST_F(CallAfterTimeoutTest, TwoResets) {
    runUntilResetWasInTime([this] {
        auto obj = callAfterTimeout(milliseconds(200));
        sleep_for(milliseconds(100));
        if (!resetTimer(obj.get())) {
            return false;
        }
        sleep_for(milliseconds(125));
        if (!resetTimer(obj.get())) {
            return false;
        }
        expectCalledAfter(milliseconds(200));
        return true;
    });
}
