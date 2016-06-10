#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_

#include <cpp-utils/system/time.h>
#include <cpp-utils/system/stat.h>

class TimestampTestUtils {
public:
    using TimestampUpdateBehavior = std::function<void (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation)>;

    TimestampUpdateBehavior ExpectUpdatesAccessTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(statBeforeOperation);
        UNUSED(timeBeforeOperation);
        UNUSED(timeAfterOperation);
        EXPECT_LE(timeBeforeOperation, statAfterOperation.st_atim);
        EXPECT_GE(timeAfterOperation, statAfterOperation.st_atim);
    };

    TimestampUpdateBehavior ExpectDoesntUpdateAccessTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(timeBeforeOperation);
        UNUSED(timeAfterOperation);
        EXPECT_EQ(statBeforeOperation.st_atim, statAfterOperation.st_atim);
    };

    TimestampUpdateBehavior ExpectUpdatesModificationTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(statBeforeOperation);
        EXPECT_LE(timeBeforeOperation, statAfterOperation.st_mtim);
        EXPECT_GE(timeAfterOperation, statAfterOperation.st_mtim);
    };

    TimestampUpdateBehavior ExpectDoesntUpdateModificationTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(timeBeforeOperation);
        UNUSED(timeAfterOperation);
        EXPECT_EQ(statBeforeOperation.st_mtim, statAfterOperation.st_mtim);
    };

    TimestampUpdateBehavior ExpectUpdatesMetadataTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(statBeforeOperation);
        EXPECT_LE(timeBeforeOperation, statAfterOperation.st_ctim);
        EXPECT_GE(timeAfterOperation, statAfterOperation.st_ctim);
    };

    TimestampUpdateBehavior ExpectDoesntUpdateMetadataTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        UNUSED(timeBeforeOperation);
        UNUSED(timeAfterOperation);
        EXPECT_EQ(statBeforeOperation.st_ctim, statAfterOperation.st_ctim);
    };

    TimestampUpdateBehavior ExpectDoesntUpdateAnyTimestamps = [this] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
        ExpectDoesntUpdateAccessTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
        ExpectDoesntUpdateModificationTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
        ExpectDoesntUpdateMetadataTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    };

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(std::function<struct stat()> stat, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        struct stat oldStat = stat();
        ensureNodeTimestampsAreOld(oldStat);
        timespec timeBeforeOperation = cpputils::time::now();
        operation();
        timespec timeAfterOperation = cpputils::time::now();
        struct stat newStat = stat();
        for (auto behaviorCheck : behaviorChecks) {
            behaviorCheck(oldStat, newStat, timeBeforeOperation, timeAfterOperation);
        }
    }

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const fspp::OpenFile &node, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        return EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS([&node](){return stat(node);}, operation, behaviorChecks);
    }

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const fspp::Node &node, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        return EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS([&node](){return stat(node);}, operation, behaviorChecks);
    }

    template<typename NodeType>
    void EXPECT_ACCESS_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_atim);
        EXPECT_GE(upperBound, stat(node).st_atim);
    }

    template<typename NodeType>
    void EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_mtim);
        EXPECT_GE(upperBound, stat(node).st_mtim);
    }

    template<typename NodeType>
    void EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_ctim);
        EXPECT_GE(upperBound, stat(node).st_ctim);
    }

    template<typename NodeType>
    static struct stat stat(const NodeType &node) {
        struct stat st;
        node.stat(&st);
        return st;
    }

    timespec xSecondsAgo(int sec) {
        timespec result = cpputils::time::now();
        result.tv_sec -= sec;
        return result;
    }

    void ensureNodeTimestampsAreOld(const struct stat &nodeStat) {
        waitUntilClockProgresses();
        EXPECT_LT(nodeStat.st_atim, cpputils::time::now());
        EXPECT_LT(nodeStat.st_mtim, cpputils::time::now());
        EXPECT_LT(nodeStat.st_ctim, cpputils::time::now());
    }

private:

    void waitUntilClockProgresses() {
        auto start = cpputils::time::now();
        while (start == cpputils::time::now()) {
            // busy waiting is the fastest, we only have to wait for a nanosecond increment.
        }
    }
};

#endif
