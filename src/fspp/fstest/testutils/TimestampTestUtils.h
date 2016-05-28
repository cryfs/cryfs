#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_TIMESTAMPTESTUTILS_H_

#include <cpp-utils/system/time.h>

class TimestampTestUtils {
public:
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

    void EXPECT_OPERATION_DOESNT_UPDATE_ACCESS_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec oldTime = stat(node).st_atim;
        operation();
        timespec newTime = stat(node).st_atim;
        EXPECT_EQ(oldTime, newTime);
    }

    void EXPECT_OPERATION_DOESNT_UPDATE_MODIFICATION_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec oldTime = stat(node).st_mtim;
        operation();
        timespec newTime = stat(node).st_mtim;
        EXPECT_EQ(oldTime, newTime);
    }

    void EXPECT_OPERATION_DOESNT_UPDATE_METADATACHANGE_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec oldTime = stat(node).st_ctim;
        operation();
        timespec newTime = stat(node).st_ctim;
        EXPECT_EQ(oldTime, newTime);
    }

    void EXPECT_OPERATION_UPDATES_ACCESS_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec lowerBound = cpputils::time::now();
        operation();
        timespec upperBound = cpputils::time::now();
        EXPECT_ACCESS_TIMESTAMP_BETWEEN(lowerBound, upperBound, node);
    }

    void EXPECT_OPERATION_UPDATES_MODIFICATION_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec lowerBound = cpputils::time::now();
        operation();
        timespec upperBound = cpputils::time::now();
        EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(lowerBound, upperBound, node);
    }

    void EXPECT_OPERATION_UPDATES_METADATACHANGE_TIMESTAMP(const fspp::Node &node, std::function<void()> operation) {
        ensureNodeTimestampsAreOld(node);
        timespec lowerBound = cpputils::time::now();
        operation();
        timespec upperBound = cpputils::time::now();
        EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, node);
    }

    void EXPECT_OPERATION_DOESNT_UPDATE_TIMESTAMPS(const fspp::Node &node, std::function<void()> operation) {
        EXPECT_OPERATION_DOESNT_UPDATE_ACCESS_TIMESTAMP(node, operation);
        EXPECT_OPERATION_DOESNT_UPDATE_MODIFICATION_TIMESTAMP(node, operation);
        EXPECT_OPERATION_DOESNT_UPDATE_METADATACHANGE_TIMESTAMP(node, operation);
    }

    struct stat stat(const fspp::Node &node) {
        struct stat st;
        node.stat(&st);
        return st;
    }

    timespec xSecondsAgo(int sec) {
        timespec result = cpputils::time::now();
        result.tv_sec -= sec;
        return result;
    }

private:

    void ensureNodeTimestampsAreOld(const fspp::Node &node) {
        waitUntilClockProgresses();
        EXPECT_LT(stat(node).st_atim, cpputils::time::now());
        EXPECT_LT(stat(node).st_mtim, cpputils::time::now());
        EXPECT_LT(stat(node).st_ctim, cpputils::time::now());
    }

    void waitUntilClockProgresses() {
        auto start = cpputils::time::now();
        while (start == cpputils::time::now()) {
            // busy waiting is the fastest, we only have to wait for a nanosecond increment.
        }
    }
};

#endif
