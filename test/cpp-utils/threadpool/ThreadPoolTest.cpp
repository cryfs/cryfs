#include <gtest/gtest.h>
#include "cpp-utils/threadpool/ThreadPool.h"

using cpputils::ThreadPool;

class ThreadPoolTest : public ::testing::Test {
};

TEST_F(ThreadPoolTest, onethread) {
    ThreadPool pool(1);
    auto value = pool.run(std::packaged_task<int ()>([] {
        return 5;
    }));
    EXPECT_EQ(5, value.get());
}

TEST_F(ThreadPoolTest, fivethreads) {
    ThreadPool pool(5);
    auto value = pool.run(std::packaged_task<int ()>([] {
        return 5;
    }));
    EXPECT_EQ(5, value.get());
}

TEST_F(ThreadPoolTest, isAsync) {
    ThreadPool pool(1);
    bool finished = false;
    auto value = pool.run(std::packaged_task<int ()>([&finished] {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        finished = true;
        return 5;
    }));
    EXPECT_FALSE(finished);
    EXPECT_EQ(5, value.get());
    EXPECT_TRUE(finished);
}

// Task 2 waits for task 1. This way, it is ensured that ThreadPool is not running them sequentially
TEST_F(ThreadPoolTest, runsInParallel_1) {
    ThreadPool pool(5);
    bool first_finished = false;
    auto value1 = pool.run(std::packaged_task<int ()>([&first_finished] {
        first_finished = true;
        return 5;
    }));
    auto value2 = pool.run(std::packaged_task<int ()>([&first_finished] {
        while (!first_finished) {}
        return 6;
    }));
    EXPECT_EQ(5, value1.get());
    EXPECT_EQ(6, value2.get());
}

// Task 1 waits for task 2. This way, it is ensured that ThreadPool is not running them sequentially reverse.
TEST_F(ThreadPoolTest, runsInParallel_2) {
    ThreadPool pool(5);
    bool second_finished = false;
    auto value1 = pool.run(std::packaged_task<int ()>([&second_finished] {
        while (!second_finished) {}
        return 5;
    }));
    auto value2 = pool.run(std::packaged_task<int ()>([&second_finished] {
        second_finished=true;
        return 6;
    }));
    EXPECT_EQ(5, value1.get());
    EXPECT_EQ(6, value2.get());
}