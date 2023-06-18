#include <gtest/gtest.h>
#include <cpp-utils/system/time.h>
#include <chrono>
#include <thread>

using cpputils::time::now;

namespace {

uint64_t _to_nanos(struct timespec time) {
    constexpr uint64_t nanos = UINT64_C(1000000000);
    return time.tv_sec * nanos + time.tv_nsec;
}
}

TEST(TimeTest, DoesntCrash) {
    now();
}

TEST(TimeTest, IsLaterThanYear2010) {
    struct timespec current_time = now();
    constexpr time_t year_2010_timestamp = 1262304000;
    EXPECT_LT(year_2010_timestamp, current_time.tv_sec);
}

TEST(TimeTest, IsNondecreasing) {
    uint64_t time1 = _to_nanos(now());
    uint64_t time2 = _to_nanos(now());
    EXPECT_LE(time1, time2);
}

TEST(TimeTest, IsIncreasedAfterPause) {
    uint64_t time1 = _to_nanos(now());
    std::this_thread::sleep_for(std::chrono::milliseconds(10));
    uint64_t time2 = _to_nanos(now());
    EXPECT_LT(time1, time2);
}

constexpr struct timespec time1 {1262304000, 000000000};
constexpr struct timespec time2 {1262304000, 000000001};
constexpr struct timespec time3 {1262304000, 100000000};
constexpr struct timespec time4 {1262304001, 000000001};

TEST(TimeTest, LessThan) {
    EXPECT_FALSE(time1 < time1);
    EXPECT_TRUE(time1 < time2);
    EXPECT_TRUE(time1 < time3);
    EXPECT_TRUE(time1 < time4);
    EXPECT_FALSE(time2 < time1);
    EXPECT_FALSE(time2 < time2);
    EXPECT_TRUE(time2 < time3);
    EXPECT_TRUE(time2 < time4);
    EXPECT_FALSE(time3 < time1);
    EXPECT_FALSE(time3 < time2);
    EXPECT_FALSE(time3 < time3);
    EXPECT_TRUE(time3 < time4);
    EXPECT_FALSE(time4 < time1);
    EXPECT_FALSE(time4 < time2);
    EXPECT_FALSE(time4 < time3);
    EXPECT_FALSE(time4 < time4);
}

TEST(TimeTest, GreaterThan) {
    EXPECT_FALSE(time1 > time1);
    EXPECT_FALSE(time1 > time2);
    EXPECT_FALSE(time1 > time3);
    EXPECT_FALSE(time1 > time4);
    EXPECT_TRUE(time2 > time1);
    EXPECT_FALSE(time2 > time2);
    EXPECT_FALSE(time2 > time3);
    EXPECT_FALSE(time2 > time4);
    EXPECT_TRUE(time3 > time1);
    EXPECT_TRUE(time3 > time2);
    EXPECT_FALSE(time3 > time3);
    EXPECT_FALSE(time3 > time4);
    EXPECT_TRUE(time4 > time1);
    EXPECT_TRUE(time4 > time2);
    EXPECT_TRUE(time4 > time3);
    EXPECT_FALSE(time4 > time4);
}

TEST(TimeTest, LessEquals) {
    EXPECT_TRUE(time1 <= time1);
    EXPECT_TRUE(time1 <= time2);
    EXPECT_TRUE(time1 <= time3);
    EXPECT_TRUE(time1 <= time4);
    EXPECT_FALSE(time2 <= time1);
    EXPECT_TRUE(time2 <= time2);
    EXPECT_TRUE(time2 <= time3);
    EXPECT_TRUE(time2 <= time4);
    EXPECT_FALSE(time3 <= time1);
    EXPECT_FALSE(time3 <= time2);
    EXPECT_TRUE(time3 <= time3);
    EXPECT_TRUE(time3 <= time4);
    EXPECT_FALSE(time4 <= time1);
    EXPECT_FALSE(time4 <= time2);
    EXPECT_FALSE(time4 <= time3);
    EXPECT_TRUE(time4 <= time4);
}

TEST(TimeTest, GreaterEquals) {
    EXPECT_TRUE(time1 >= time1);
    EXPECT_FALSE(time1 >= time2);
    EXPECT_FALSE(time1 >= time3);
    EXPECT_FALSE(time1 >= time4);
    EXPECT_TRUE(time2 >= time1);
    EXPECT_TRUE(time2 >= time2);
    EXPECT_FALSE(time2 >= time3);
    EXPECT_FALSE(time2 >= time4);
    EXPECT_TRUE(time3 >= time1);
    EXPECT_TRUE(time3 >= time2);
    EXPECT_TRUE(time3 >= time3);
    EXPECT_FALSE(time3 >= time4);
    EXPECT_TRUE(time4 >= time1);
    EXPECT_TRUE(time4 >= time2);
    EXPECT_TRUE(time4 >= time3);
    EXPECT_TRUE(time4 >= time4);
}

TEST(TimeTest, Equals) {
    EXPECT_TRUE(time1 == time1);
    EXPECT_FALSE(time1 == time2);
    EXPECT_FALSE(time1 == time3);
    EXPECT_FALSE(time1 == time4);
    EXPECT_FALSE(time2 == time1);
    EXPECT_TRUE(time2 == time2);
    EXPECT_FALSE(time2 == time3);
    EXPECT_FALSE(time2 == time4);
    EXPECT_FALSE(time3 == time1);
    EXPECT_FALSE(time3 == time2);
    EXPECT_TRUE(time3 == time3);
    EXPECT_FALSE(time3 == time4);
    EXPECT_FALSE(time4 == time1);
    EXPECT_FALSE(time4 == time2);
    EXPECT_FALSE(time4 == time3);
    EXPECT_TRUE(time4 == time4);
}

TEST(TimeTest, NotEquals) {
    EXPECT_FALSE(time1 != time1);
    EXPECT_TRUE(time1 != time2);
    EXPECT_TRUE(time1 != time3);
    EXPECT_TRUE(time1 != time4);
    EXPECT_TRUE(time2 != time1);
    EXPECT_FALSE(time2 != time2);
    EXPECT_TRUE(time2 != time3);
    EXPECT_TRUE(time2 != time4);
    EXPECT_TRUE(time3 != time1);
    EXPECT_TRUE(time3 != time2);
    EXPECT_FALSE(time3 != time3);
    EXPECT_TRUE(time3 != time4);
    EXPECT_TRUE(time4 != time1);
    EXPECT_TRUE(time4 != time2);
    EXPECT_TRUE(time4 != time3);
    EXPECT_FALSE(time4 != time4);
}
