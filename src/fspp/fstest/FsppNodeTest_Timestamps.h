#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_TIMESTAMPS_H_

#include "testutils/FsppNodeTest.h"
#include "../fs_interface/FuseErrnoException.h"
#include "testutils/TimestampTestUtils.h"
#include <cpp-utils/system/stat.h>


template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:

    void Test_Create() {
        this->testBuilder().withAnyAtimeConfig([&] {
            timespec lowerBound = cpputils::time::now();
            auto node = this->CreateNode("/mynode");
            timespec upperBound = cpputils::time::now();
            this->EXPECT_ACCESS_TIMESTAMP_BETWEEN(lowerBound, upperBound, *node);
            this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN(lowerBound, upperBound, *node);
            this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *node);
        });
    }

    void Test_Stat() {
        auto operation = [this] {
            auto node = this->CreateNode("/mynode");
            return [node = std::move(node)] {
                node->stat();
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Chmod() {
        auto operation = [this] {
            auto node = this->CreateNode("/mynode");
            fspp::mode_t mode = this->stat(*node).mode;
            return [mode, node = std::move(node)] {
                node->chmod(mode);
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Chown() {
        auto operation = [this] {
            auto node = this->CreateNode("/mynode");
            fspp::uid_t uid = this->stat(*node).uid;
            fspp::gid_t gid = this->stat(*node).gid;
            return [uid, gid, node = std::move(node)] {
                node->chown(uid, gid);
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Access() {
        auto operation = [this] {
            auto node = this->CreateNode("/mynode");
            return [node = std::move(node)] {
                node->access(0);
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Error_TargetParentDirDoesntExist() {
        auto operation = [this] {
            auto node = this->CreateNode("/oldname");
            return [node = std::move(node)] {
                try {
                    node->rename("/notexistingdir/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(ENOENT, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Error_TargetParentDirIsFile() {
        auto operation = [this] {
            auto node = this->CreateNode("/oldname");
            this->CreateFile("/somefile");
            return [node = std::move(node)] {
                try {
                    node->rename("/somefile/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Error_RootDir() {
        // TODO Re-enable this test once the root dir stores timestamps correctly
        /*
        auto operation = [this] {
            auto root = this->Load("/");
            return [root = std::move(root)] {
                try {
                    root->rename("/newname");
                    EXPECT_TRUE(false); // expect throws
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(EBUSY, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
        */
    }

    void Test_Rename_InRoot() {
        auto operation = [this] {
            auto node = this->CreateNode("/oldname");
            return [node = std::move(node)] {
                node->rename("/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_InNested() {
        auto operation = [this] {
            this->CreateDir("/mydir");
            auto node = this->CreateNode("/mydir/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/mydir/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_RootToNested_SameName() {
        auto operation = [this] {
            this->CreateDir("/mydir");
            auto node = this->CreateNode("/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir/oldname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/mydir/oldname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_RootToNested_NewName() {
        auto operation = [this] {
            this->CreateDir("/mydir");
            auto node = this->CreateNode("/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/mydir/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_NestedToRoot_SameName() {
        auto operation = [this] {
            this->CreateDir("/mydir");
            auto node = this->CreateNode("/mydir/oldname");
            return [node = std::move(node)] {
                node->rename("/oldname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/oldname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_NestedToRoot_NewName() {
        auto operation = [this] {
            this->CreateDir("/mydir");
            auto node = this->CreateNode("/mydir/oldname");
            return [node = std::move(node)] {
                node->rename("/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir/oldname", "/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_NestedToNested_SameName() {
        auto operation = [this] {
            this->CreateDir("/mydir1");
            this->CreateDir("/mydir2");
            auto node = this->CreateNode("/mydir1/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir2/oldname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/oldname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_NestedToNested_NewName() {
        auto operation = [this] {
            this->CreateDir("/mydir1");
            this->CreateDir("/mydir2");
            auto node = this->CreateNode("/mydir1/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir2/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_ToItself() {
        auto operation = [this] {
            auto node = this->CreateNode("/oldname");
            return [node = std::move(node)] {
                node->rename("/oldname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/oldname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_Overwrite_InSameDir() {
        auto operation = [this] {
            auto node = this->CreateNode("/oldname");
            this->CreateNode("/newname");
            return [node = std::move(node)] {
                node->rename("/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", "/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_Overwrite_InDifferentDir() {
        auto operation = [this] {
            this->CreateDir("/mydir1");
            this->CreateDir("/mydir2");
            this->CreateNode("/mydir2/newname");
            auto node = this->CreateNode("/mydir1/oldname");
            return [node = std::move(node)] {
                node->rename("/mydir2/newname");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", "/mydir2/newname", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    void Test_Rename_Overwrite_Error_DirWithFile_InSameDir() {
        auto operation = [this] {
            this->CreateFile("/oldname");
            this->CreateDir("/newname");
            auto node = this->Load("/oldname");
            return [node = std::move(node)] {
                try {
                    node->rename("/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(EISDIR, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Overwrite_Error_DirWithFile_InDifferentDir() {
        auto operation = [this] {
            this->CreateDir("/mydir1");
            this->CreateDir("/mydir2");
            this->CreateFile("/mydir1/oldname");
            this->CreateDir("/mydir2/newname");
            auto node = this->Load("/mydir1/oldname");
            return [node = std::move(node)] {
                try {
                    node->rename("/mydir2/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(EISDIR, e.getErrno());//Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Overwrite_Error_FileWithDir_InSameDir() {
        auto operation = [this] {
            this->CreateDir("/oldname");
            this->CreateFile("/newname");
            auto node = this->Load("/oldname");
            return [node = std::move(node)] {
                try {
                    node->rename("/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Rename_Overwrite_Error_FileWithDir_InDifferentDir() {
        auto operation = [this] {
            this->CreateDir("/mydir1");
            this->CreateDir("/mydir2");
            this->CreateDir("/mydir1/oldname");
            this->CreateFile("/mydir2/newname");
            auto node = this->Load("/mydir1/oldname");
            return [node = std::move(node)] {
                try {
                    node->rename("/mydir2/newname");
                    EXPECT_TRUE(false); // expect rename to fail
                } catch (const fspp::fuse::FuseErrnoException &e) {
                    EXPECT_EQ(ENOTDIR, e.getErrno()); //Rename fails, everything is ok.
                }
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir1/oldname", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
        });
    }

    void Test_Utimens() {
        this->testBuilder().withAnyAtimeConfig([&] {
            auto node = this->CreateNode("/mynode");
            timespec atime = this->xSecondsAgo(100);
            timespec mtime = this->xSecondsAgo(200);
            auto operation = [atime, mtime, &node] {
                node->utimens(atime, mtime);
            };
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {
                this->ExpectUpdatesMetadataTimestamp
            });
            EXPECT_EQ(atime, this->stat(*node).atime);
            EXPECT_EQ(mtime, this->stat(*node).mtime);
        });
    }
};

REGISTER_NODE_TEST_SUITE(FsppNodeTest_Timestamps,
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
