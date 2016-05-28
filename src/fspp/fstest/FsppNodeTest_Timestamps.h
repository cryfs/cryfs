#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_

#include "testutils/FsppNodeTest.h"
#include "../fuse/FuseErrnoException.h"
#include "testutils/TimestampTestUtils.h"

using namespace cpputils::time;
using std::function;

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils {
public:

    void Test_Create() {
        timespec lowerBound = now();
        auto node = this->CreateNode("/mynode");
        timespec upperBound = now();
        EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *node);
        EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *node);
        EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *node);
    }

    void Test_Stat() {
        auto node = this->CreateNode("/mynode");
        auto operation = [&node] () {
            struct stat st;
            node->stat(&st);
        };
        EXPECT_OPERATION_DOESNT_UPDATE_TIMESTAMPS(*node, operation);
    }

    void Test_Chmod() {
        auto node = this->CreateNode("/mynode");
        mode_t mode = stat(*node).st_mode;
        auto operation = [&node, mode] () {
            node->chmod(mode);
        };
        EXPECT_OPERATION_DOESNT_UPDATE_ACCESS_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_DOESNT_UPDATE_MODIFICATION_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_UPDATES_METADATACHANGE_TIMESTAMP(*node, operation);
    }

    void Test_Chown() {
        auto node = this->CreateNode("/mynode");
        uid_t uid = stat(*node).st_uid;
        gid_t gid = stat(*node).st_gid;
        auto operation = [&node, uid, gid] () {
            node->chown(uid, gid);
        };
        EXPECT_OPERATION_DOESNT_UPDATE_ACCESS_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_DOESNT_UPDATE_MODIFICATION_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_UPDATES_METADATACHANGE_TIMESTAMP(*node, operation);
    }

    void Test_Access() {
        auto node = this->CreateNode("/mynode");
        auto operation = [&node] () {
            node->access(0);
        };
        EXPECT_OPERATION_DOESNT_UPDATE_TIMESTAMPS(*node, operation);
    }

    void Test_Rename() {
        auto node = this->CreateNode("/mynode");
        auto operation = [&node] () {
            node->rename("newnodename");
        };
        EXPECT_OPERATION_DOESNT_UPDATE_ACCESS_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_DOESNT_UPDATE_MODIFICATION_TIMESTAMP(*node, operation);
        EXPECT_OPERATION_UPDATES_METADATACHANGE_TIMESTAMP(*node, operation);
    }

    // TODO Other rename cases (e.g. failed renames/error paths, moving to different dir, ...) from FsppNodeTest_Rename

    void Test_Utimens() {
        auto node = this->CreateNode("/mynode");
        timespec atime = xSecondsAgo(100);
        timespec mtime = xSecondsAgo(200);
        auto operation = [&node, atime, mtime] () {
            node->utimens(atime, mtime);
        };
        EXPECT_OPERATION_UPDATES_METADATACHANGE_TIMESTAMP(*node, operation);
        EXPECT_EQ(atime, stat(*node).st_atim);
        EXPECT_EQ(mtime, stat(*node).st_mtim);
    }
};

REGISTER_NODE_TEST_CASE(FsppNodeTest_Timestamps,
    Create,
    Stat,
    Chmod,
    Chown,
    Access,
    Rename,
    Utimens
);

#endif
