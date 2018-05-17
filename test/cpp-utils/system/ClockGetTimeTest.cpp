#include <gtest/gtest.h>
#include <cpp-utils/system/clock_gettime.h>
#include <chrono>
#include <thread>

namespace {
struct timespec _gettime() {
    struct timespec current_time;
    int res = clock_gettime(CLOCK_REALTIME, &current_time);
    EXPECT_EQ(0, res);
    return current_time;
}

uint64_t _to_nanos(struct timespec time) {
    constexpr uint64_t nanos = UINT64_C(1000000000);
    return time.tv_sec * nanos + time.tv_nsec;
}
}

TEST(ClockGetTimeTest, DoesntCrash) {
    _gettime();
}

TEST(ClockGetTimeTest, IsLaterThanYear2010) {
    struct timespec current_time = _gettime();
    EXPECT_LT(1262304000, current_time.tv_sec);
}

TEST(ClockGetTimeTest, IsNondecreasing) {
    uint64_t time1 = _to_nanos(_gettime());
    uint64_t time2 = _to_nanos(_gettime());
    EXPECT_LE(time1, time2);
}

TEST(ClockGetTimeTest, IsIncreasedAfterPause) {
    uint64_t time1 = _to_nanos(_gettime());
    std::this_thread::sleep_for(std::chrono::milliseconds(10));
    uint64_t time2 = _to_nanos(_gettime());
    EXPECT_LT(time1, time2);
}
