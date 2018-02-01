#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_

#include "testutils/FsppNodeTest.h"
#include "../fuse/FuseErrnoException.h"
#include "testutils/TimestampTestUtils.h"
#include <cpp-utils/system/stat.h>

using namespace cpputils::time;
using std::function;

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:

    void Test_Create() {
        timespec lowerBound = now();
        auto node = this->CreateNode("/mynode");
        timespec upperBound = now();
        this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *node);
        this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *node);
        this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *node);
    }

    void Test_Stat() {
        this->CreateNode("/mynode");
        auto operation = [this] () {
            struct stat st{};
            this->Load("/mynode")->stat(&st);
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Chmod() {
        auto node = this->CreateNode("/mynode");
        mode_t mode = this->stat(*node).st_mode;
        cpputils::destruct(std::move(node));
        auto operation = [this, mode] () {
            this->Load("/mynode")->chmod(mode);
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Chown() {
        auto node = this->CreateNode("/mynode");
        uid_t uid = this->stat(*node).st_uid;
        gid_t gid = this->stat(*node).st_gid;
        cpputils::destruct(std::move(node));
        auto operation = [this, uid, gid] () {
            this->Load("/mynode")->chown(uid, gid);
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Access() {
        this->CreateNode("/mynode");
        auto operation = [this] () {
            this->Load("/mynode")->access(0);
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Error_TargetParentDirDoesntExist() {
        this->CreateNode("/oldname");
        auto operation = [this] () {
            auto node = this->Load("/oldname");
            try {
                node->rename("/notexistingdir/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(ENOENT, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Error_TargetParentDirIsFile() {
        this->CreateNode("/oldname");
        this->CreateFile("/somefile");
        auto operation = [this] () {
            auto node = this->Load("/oldname");
            try {
                node->rename("/somefile/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Error_RootDir() {
        // TODO Re-enable this test once the root dir stores timestamps correctly
        /*
        auto operation = [this] () {
            auto root = this->Load("/");
            try {
                root->rename("/newname");
                EXPECT_TRUE(false); // expect throws
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(EBUSY, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
             this->ExpectDoesntUpdateAnyTimestamps
         });
         */
    }

    void Test_Rename_InRoot() {
        this->CreateNode("/oldname");
        auto operation = [this] () {
            this->Load("/oldname")->rename("/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_InNested() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/oldname");
        auto operation = [this] () {
            this->Load("/mydir/oldname")->rename("/mydir/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/mydir/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_RootToNested_SameName() {
        this->CreateDir("/mydir");
        this->CreateNode("/oldname");
        auto operation = [this] () {
            this->Load("/oldname")->rename("/mydir/oldname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/mydir/oldname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_RootToNested_NewName() {
        this->CreateDir("/mydir");
        this->CreateNode("/oldname");
        auto operation = [this] () {
            this->Load("/oldname")->rename("/mydir/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/mydir/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_NestedToRoot_SameName() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/oldname");
        auto operation = [this] () {
            this->Load("/mydir/oldname")->rename("/oldname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/oldname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_NestedToRoot_NewName() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/oldname");
        auto operation = [this] () {
            this->Load("/mydir/oldname")->rename("/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_NestedToNested_SameName() {
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        this->CreateNode("/mydir1/oldname");
        auto operation = [this] () {
            this->Load("/mydir1/oldname")->rename("/mydir2/oldname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/oldname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_NestedToNested_NewName() {
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        this->CreateNode("/mydir1/oldname");
        auto operation = [this] () {
            this->Load("/mydir1/oldname")->rename("/mydir2/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_ToItself() {
        this->CreateNode("/oldname");
        auto operation = [this] () {
            this->Load("/oldname")->rename("/oldname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/oldname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_Overwrite_InSameDir() {
        this->CreateNode("/oldname");
        this->CreateNode("/newname");
        auto operation = [this] () {
            this->Load("/oldname")->rename("/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_Overwrite_InDifferentDir() {
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        this->CreateNode("/mydir2/newname");
        this->CreateNode("/mydir1/oldname");
        auto operation = [this] () {
            this->Load("/mydir1/oldname")->rename("/mydir2/newname");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/newname", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectDoesntUpdateModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    void Test_Rename_Overwrite_Error_DirWithFile_InSameDir() {
        this->CreateFile("/oldname");
        this->CreateDir("/newname");
        auto operation = [this] () {
            auto node = this->Load("/oldname");
            try {
                node->rename("/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(EISDIR, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Overwrite_Error_DirWithFile_InDifferentDir() {
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        this->CreateFile("/mydir1/oldname");
        this->CreateDir("/mydir2/newname");
        auto operation = [this] () {
            auto node = this->Load("/mydir1/oldname");
            try {
                node->rename("/mydir2/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(EISDIR, e.getErrno());//Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Overwrite_Error_FileWithDir_InSameDir() {
        this->CreateDir("/oldname");
        this->CreateFile("/newname");
        auto operation = [this] () {
            auto node = this->Load("/oldname");
            try {
                node->rename("/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Rename_Overwrite_Error_FileWithDir_InDifferentDir() {
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        this->CreateDir("/mydir1/oldname");
        this->CreateFile("/mydir2/newname");
        auto operation = [this] () {
            auto node = this->Load("/mydir1/oldname");
            try {
                node->rename("/mydir2/newname");
                EXPECT_TRUE(false); // expect rename to fail
            } catch (const fspp::fuse::FuseErrnoException &e) {
                EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
            }
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", operation, {
            this->ExpectDoesntUpdateAnyTimestamps
        });
    }

    void Test_Utimens() {
        this->CreateNode("/mynode");
        timespec atime = this->xSecondsAgo(100);
        timespec mtime = this->xSecondsAgo(200);
        auto operation = [this, atime, mtime] () {
            this->Load("/mynode")->utimens(atime, mtime);
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
            this->ExpectUpdatesMetadataTimestamp
        });
        auto node = this->Load("/mynode");
        EXPECT_EQ(atime, this->stat(*node).st_atim);
        EXPECT_EQ(mtime, this->stat(*node).st_mtim);
    }
};

REGISTER_NODE_TEST_CASE(FsppNodeTest_Timestamps,
    Create,
    Stat,
    Chmod,
    Chown,
    Access,
    Rename_Error_TargetParentDirDoesntExist,
    Rename_Error_TargetParentDirIsFile,
    Rename_Error_RootDir,
    Rename_InRoot,
    Rename_InNested,
    Rename_RootToNested_SameName,
    Rename_RootToNested_NewName,
    Rename_NestedToRoot_SameName,
    Rename_NestedToRoot_NewName,
    Rename_NestedToNested_SameName,
    Rename_NestedToNested_NewName,
    Rename_ToItself,
    Rename_Overwrite_InSameDir,
    Rename_Overwrite_InDifferentDir,
    Rename_Overwrite_Error_DirWithFile_InSameDir,
    Rename_Overwrite_Error_DirWithFile_InDifferentDir,
    Rename_Overwrite_Error_FileWithDir_InSameDir,
    Rename_Overwrite_Error_FileWithDir_InDifferentDir,
    Utimens
);

#endif
