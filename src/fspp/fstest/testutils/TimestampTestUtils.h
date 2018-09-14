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
    using TimestampUpdateBehavior = std::function<void (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation)>;

    static TimestampUpdateBehavior ExpectUpdatesAccessTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAccessTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesModificationTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateModificationTimestamp;
    static TimestampUpdateBehavior ExpectUpdatesMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateMetadataTimestamp;
    static TimestampUpdateBehavior ExpectDoesntUpdateAnyTimestamps;

    void EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(std::function<fspp::Node::stat_info()> statOld, std::function<fspp::Node::stat_info()> statNew, std::function<void()> operation, std::initializer_list<TimestampUpdateBehavior> behaviorChecks) {
        auto oldStat = statOld();
        ensureNodeTimestampsAreOld(oldStat);
        timespec timeBeforeOperation = cpputils::time::now();
        operation();
        timespec timeAfterOperation = cpputils::time::now();
        auto newStat = statNew();
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
        EXPECT_LE(lowerBound, stat(node).atime);
        EXPECT_GE(upperBound, stat(node).atime);
    }

    void EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const fspp::Node &node) {
        EXPECT_LE(lowerBound, stat(node).mtime);
        EXPECT_GE(upperBound, stat(node).mtime);
    }

    void EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(timespec lowerBound, timespec upperBound, const fspp::Node &node) {
        EXPECT_LE(lowerBound, stat(node).ctime);
        EXPECT_GE(upperBound, stat(node).ctime);
    }

    static fspp::Node::stat_info stat(const fspp::Node &node) {
        return node.stat();
    }

    static fspp::Node::stat_info stat(const fspp::OpenFile &openFile) {
        return openFile.stat();
    }

    timespec xSecondsAgo(int sec) {
        timespec result = cpputils::time::now();
        result.tv_sec -= sec;
        return result;
    }

    void ensureNodeTimestampsAreOld(const fspp::Node::stat_info &nodeStat) {
        waitUntilClockProgresses();
        EXPECT_LT(nodeStat.atime, cpputils::time::now());
        EXPECT_LT(nodeStat.mtime, cpputils::time::now());
        EXPECT_LT(nodeStat.ctime, cpputils::time::now());
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
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.atime);
    EXPECT_GE(timeAfterOperation, statAfterOperation.atime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateAccessTimestamp =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.atime, statAfterOperation.atime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectUpdatesModificationTimestamp =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.mtime);
    EXPECT_GE(timeAfterOperation, statAfterOperation.mtime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateModificationTimestamp =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.mtime, statAfterOperation.mtime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectUpdatesMetadataTimestamp =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(statBeforeOperation);
    EXPECT_LE(timeBeforeOperation, statAfterOperation.ctime);
    EXPECT_GE(timeAfterOperation, statAfterOperation.ctime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateMetadataTimestamp =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    UNUSED(timeBeforeOperation);
    UNUSED(timeAfterOperation);
    EXPECT_EQ(statBeforeOperation.ctime, statAfterOperation.ctime);
};

template<class ConcreteFileSystemTestFixture>
typename TimestampTestUtils<ConcreteFileSystemTestFixture>::TimestampUpdateBehavior TimestampTestUtils<ConcreteFileSystemTestFixture>::ExpectDoesntUpdateAnyTimestamps =
        [] (const fspp::Node::stat_info& statBeforeOperation, const fspp::Node::stat_info& statAfterOperation, timespec timeBeforeOperation, timespec timeAfterOperation) {
    ExpectDoesntUpdateAccessTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    ExpectDoesntUpdateModificationTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
    ExpectDoesntUpdateMetadataTimestamp(statBeforeOperation, statAfterOperation, timeBeforeOperation, timeAfterOperation);
};

#endif
