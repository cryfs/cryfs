#include <gtest/gtest.h>

#include <atomic>
#include <chrono>
#include <cstring>
#include <thread>

#include "cpp-utils/data/Data.h"
#include "cpp-utils/data/DataFixture.h"
#include "cpp-utils/random/ThreadsafeRandomDataBuffer.h"

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::ThreadsafeRandomDataBuffer;

class ThreadsafeRandomDataBufferTest: public ::testing::Test {
public:
    ThreadsafeRandomDataBuffer buffer;

    Data get(size_t numBytes) {
        Data result(numBytes);
        buffer.get(result.data(), numBytes);
        return result;
    }
};

TEST_F(ThreadsafeRandomDataBufferTest, newBufferIsEmpty) {
    EXPECT_EQ(0u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, add_increasesSize) {
    buffer.add(DataFixture::generate(100));
    EXPECT_EQ(100u, buffer.size());
    buffer.add(DataFixture::generate(50));
    EXPECT_EQ(150u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, get_zeroBytes_returnsImmediatelyOnEmptyBuffer) {
    const Data got = get(0);
    EXPECT_EQ(0u, got.size());
    EXPECT_EQ(0u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, get_returnsAddedData) {
    const Data data = DataFixture::generate(100);
    buffer.add(data);
    EXPECT_EQ(data, get(100));
    EXPECT_EQ(0u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, get_partially_returnsPrefixThenRemainder) {
    const Data data = DataFixture::generate(100);
    buffer.add(data);

    const Data prefix = get(30);
    EXPECT_EQ(70u, buffer.size());
    EXPECT_EQ(0, std::memcmp(data.data(), prefix.data(), 30));

    const Data remainder = get(70);
    EXPECT_EQ(0u, buffer.size());
    EXPECT_EQ(0, std::memcmp(data.dataOffset(30), remainder.data(), 70));
}

TEST_F(ThreadsafeRandomDataBufferTest, get_afterMultipleAdds_returnsConcatenation) {
    const Data first = DataFixture::generate(100, 1);
    const Data second = DataFixture::generate(50, 2);
    buffer.add(first);
    buffer.add(second);

    const Data got = get(150);
    EXPECT_EQ(0, std::memcmp(first.data(), got.data(), 100));
    EXPECT_EQ(0, std::memcmp(second.data(), got.dataOffset(100), 50));
    EXPECT_EQ(0u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, get_onEmptyBuffer_waitsForAdd) {
    const Data data = DataFixture::generate(100);
    Data got(100);
    std::atomic<bool> getterStarted(false);
    std::atomic<bool> getterFinished(false);
    std::thread getter([&] {
        getterStarted = true;
        buffer.get(got.data(), 100);
        getterFinished = true;
    });
    while (!getterStarted.load()) {}
    // The buffer is empty, so get() has to block until data is added. Give a get() that doesn't block the time to
    // return here. A correctly blocking get() can never finish before the add() below, so this can't fail spuriously.
    std::this_thread::sleep_for(std::chrono::milliseconds(50));
    EXPECT_FALSE(getterFinished.load());

    buffer.add(data);
    getter.join();
    EXPECT_TRUE(getterFinished.load());
    EXPECT_EQ(data, got);
}

TEST_F(ThreadsafeRandomDataBufferTest, waitUntilSizeIsLessThan_returnsImmediatelyIfAlreadySmaller) {
    buffer.add(DataFixture::generate(10));
    buffer.waitUntilSizeIsLessThan(11);
    EXPECT_EQ(10u, buffer.size());
}

TEST_F(ThreadsafeRandomDataBufferTest, waitUntilSizeIsLessThan_wakesUpWhenDataIsGotten) {
    buffer.add(DataFixture::generate(100));
    std::thread waiter([&] {
        buffer.waitUntilSizeIsLessThan(50);
    });
    get(60);
    waiter.join();
    EXPECT_EQ(40u, buffer.size());
}

// Regression test: when get() can't be satisfied from the data currently in the buffer, it takes what is there and
// then waits for more. The second round used to request the full number of bytes again instead of only the ones
// still missing, overrunning the target buffer by up to the number of bytes gotten in the first round.
TEST_F(ThreadsafeRandomDataBufferTest, get_spanningTwoAdds_returnsConcatenationAndDoesNotWritePastRequestedBytes) {
    constexpr size_t FIRST_ADD = 64;
    constexpr size_t REQUESTED = 96;
    constexpr size_t STILL_MISSING_AFTER_FIRST_ADD = REQUESTED - FIRST_ADD;
    // Add more than is still missing, so that a get() taking too much in its second round has bytes to overrun with
    constexpr size_t SECOND_ADD = 128;
    constexpr size_t GUARD = 16;
    const Data first = DataFixture::generate(FIRST_ADD, 1);
    const Data second = DataFixture::generate(SECOND_ADD, 2);
    const Data guard = DataFixture::generate(GUARD, 3);

    // The guard bytes directly behind the requested range must not be touched by get()
    Data target(REQUESTED + GUARD);
    std::memcpy(target.dataOffset(REQUESTED), guard.data(), GUARD);

    buffer.add(first);
    std::thread getter([&] {
        buffer.get(target.data(), REQUESTED);
    });
    // Only add the rest once the getter has drained the first chunk. This guarantees that get() is split over
    // the two adds and has to wait in between.
    buffer.waitUntilSizeIsLessThan(1);
    buffer.add(second);
    getter.join();

    EXPECT_EQ(0, std::memcmp(first.data(), target.data(), FIRST_ADD));
    EXPECT_EQ(0, std::memcmp(second.data(), target.dataOffset(FIRST_ADD), STILL_MISSING_AFTER_FIRST_ADD));
    EXPECT_EQ(0, std::memcmp(guard.data(), target.dataOffset(REQUESTED), GUARD));
    EXPECT_EQ(SECOND_ADD - STILL_MISSING_AFTER_FIRST_ADD, buffer.size());
}
