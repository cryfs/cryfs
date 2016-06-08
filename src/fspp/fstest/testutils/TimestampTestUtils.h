#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_

#include <cpp-utils/system/time.h>
#include <cpp-utils/system/stat.h>

template<typename NodeType>
class TimestampTestUtils {
public:
    using TimestampUpdateBehavior = std::function<void (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation)>;

    static TimestampUpdateBehavior ExpectUpdatesAccessTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAccessTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesModificationTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateModificationTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAnyTimestamps;

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const NodeType &node, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        ensureNodeTimestampsAreOld(node);
        struct stat oldStat = stat(node);
        timespec timeBeforeOperation = cpputils::time::now();
        operation();
        timespec timeAfterOperation = cpputils::time::now();
        struct stat newStat = stat(node);
        for (auto behaviorCheck : behaviorChecks) {
            behaviorCheck(oldStat, newStat, timeBeforeOperation, timeAfterOperation);
        }
    }

    void EXPECT_ACCESS_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_atim);
        EXPECT_GE(upperBound, stat(node).st_atim);
    }

    void EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_mtim);
        EXPECT_GE(upperBound, stat(node).st_mtim);
    }

    void EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const NodeType &node) {
        EXPECT_LE(lowerBound, stat(node).st_ctim);
        EXPECT_GE(upperBound, stat(node).st_ctim);
    }

    struct stat stat(const NodeType &node) {
        struct stat st;
        node.stat(&st);
        return st;
    }

    timespec xSecondsAgo(int sec) {
        timespec result = cpputils::time::now();
        result.tv_sec -= sec;
        return result;
    }

    void ensureNodeTimestampsAreOld(const NodeType &node) {
        waitUntilClockProgresses();
        EXPECT_LT(stat(node).st_atim, cpputils::time::now());
        EXPECT_LT(stat(node).st_mtim, cpputils::time::now());
        EXPECT_LT(stat(node).st_ctim, cpputils::time::now());
    }

private:

    void waitUntilClockProgresses() {
        auto start = cpputils::time::now();
        while (start == cpputils::time::now()) {
            // busy waiting is the fastest, we only have to wait for a nanosecond increment.
        }
    }
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectUpdatesAccessTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_atim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_atim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectDoesntUpdateAccessTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_atim, statAfterOperation.st_atim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectUpdatesModificationTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_mtim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_mtim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectDoesntUpdateModificationTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_mtim, statAfterOperation.st_mtim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectUpdatesMetadataTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_ctim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_ctim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectDoesntUpdateMetadataTimestamp = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_ctim, statAfterOperation.st_ctim);
};

template<class NodeType>
typename TimestampTestUtils<NodeType>::TimestampUpdateBehavior TimestampTestUtils<NodeType>::ExpectDoesntUpdateAnyTimestamps = [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    TimestampTestUtils::ExpectDoesntUpdateAccessTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    TimestampTestUtils::ExpectDoesntUpdateModificationTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    TimestampTestUtils::ExpectDoesntUpdateMetadataTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
};

#endif
