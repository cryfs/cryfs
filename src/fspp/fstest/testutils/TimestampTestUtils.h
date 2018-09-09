#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_

#include <cpp-utils/system/time.h>
#include <cpp-utils/system/stat.h>
#include "FileSystemTest.h"
#include <functional>

template<class ConcreteFileSystemTestFixture>
class TimestampTestUtils : public virtual FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    using TimestampUpdateBehavior = std::function<void (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation)>;

    static TimestampUpdateBehavior ExpectUpdatesAccessTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAccessTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesModificationTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateModificationTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAnyTimestamps;

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(std::function<struct stat()> statOld, std::function<struct stat()> statNew, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        struct stat oldStat = statOld();
        ensureNodeTimestampsAreOld(oldStat);
        timespec timeBeforeOperation = cpputils::time::now();
        operation();
        timespec timeAfterOperation = cpputils::time::now();
        struct stat newStat = statNew();
        for (auto behaviorCheck : behaviorChecks) {
            behaviorCheck(oldStat, newStat, timeBeforeOperation, timeAfterOperation);
        }
    }

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const fspp::OpenFile &node, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(
            [this, &node](){return this->stat(node);},
            [this, &node](){return this->stat(node);},
            operation,
            behaviorChecks
        );
    }

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const boost::filesystem::path &oldPath, const boost::filesystem::path &newPath, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(
            [this, oldPath](){return this->stat(*this->Load(oldPath));},
            [this, newPath](){return this->stat(*this->Load(newPath));},
            operation,
            behaviorChecks
        );
    }

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(const boost::filesystem::path &path, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, path, operation, behaviorChecks);
    }

    void EXPECT_ACCESS_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const fspp::Node &node) {
        EXPECT_LE(lowerBound, stat(node).st_atim);
        EXPECT_GE(upperBound, stat(node).st_atim);
    }

    void EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const fspp::Node &node) {
        EXPECT_LE(lowerBound, stat(node).st_mtim);
        EXPECT_GE(upperBound, stat(node).st_mtim);
    }

    void EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const fspp::Node &node) {
        EXPECT_LE(lowerBound, stat(node).st_ctim);
        EXPECT_GE(upperBound, stat(node).st_ctim);
    }

    static struct stat stat(const fspp::Node &node) {
        struct stat st{};
        node.stat(&st);
        return st;
    }

    static struct stat stat(const fspp::OpenFile &openFile) {
        struct stat st{};
        openFile.stat(&st);
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

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectUpdatesAccessTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_atim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_atim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateAccessTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_atim, statAfterOperation.st_atim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectUpdatesModificationTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_mtim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_mtim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateModificationTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_mtim, statAfterOperation.st_mtim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectUpdatesMetadataTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.st_ctim);
    EXPECT_GE(timeAfterOperation, statAfterOperation.st_ctim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateMetadataTimestamp =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.st_ctim, statAfterOperation.st_ctim);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateAnyTimestamps =
        [] (struct stat statBeforeOperation, struct stat statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    ExpectDoesntUpdateAccessTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    ExpectDoesntUpdateModificationTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    ExpectDoesntUpdateMetadataTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
};

#endif
