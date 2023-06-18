#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_RENAME_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_RENAME_H_

#include "testutils/FsppNodeTest.h"
#include "../fs_interface/FuseErrnoException.h"

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Rename: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    void Test_Error_TargetParentDirDoesntExist() {
        auto node = this->CreateNode("/oldname");
        try {
            node->rename("/notexistingdir/newname");
            EXPECT_TRUE(false); // Expect it throws an exception
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOENT, e.getErrno());
        }
        //Old node should still exist
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_Error_TargetParentDirIsFile() {
        this->CreateNode("/oldname");
        this->CreateFile("/somefile");
        try {
            this->Load("/somefile")->rename("/somefile/newname");
            EXPECT_TRUE(false); // Expect it throws an exception
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        //Nodes should still exist
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/somefile"));
    }

    void Test_Error_RootDir() {
        auto rootDirNode = this->Load("/");
        try {
            rootDirNode->rename("/newname");
            EXPECT_TRUE(false); // expect throws
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EBUSY, e.getErrno());
        }
    }

    void Test_InRoot() {
        auto node = this->CreateNode("/oldname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_InNested() {
        this->CreateDir("/mydir");
        auto node = this->CreateNode("/mydir/oldname");
        node->rename("/mydir/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/newname"));
    }

    void Test_RootToNested_SameName() {
        this->CreateDir("/mydir");
        auto node = this->CreateNode("/oldname");
        node->rename("/mydir/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/oldname"));
    }

    void Test_RootToNested_NewName() {
        this->CreateDir("/mydir");
        auto node = this->CreateNode("/oldname");
        node->rename("/mydir/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/newname"));
    }

    void Test_NestedToRoot_SameName() {
        this->CreateDir("/mydir");
        auto node = this->CreateNode("/mydir/oldname");
        node->rename("/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_NestedToRoot_NewName() {
        this->CreateDir("/mydir");
        auto node = this->CreateNode("/mydir/oldname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_NestedToNested_SameName() {
        this->CreateDir("/mydir");
        this->CreateDir("/mydir2");
        auto node = this->CreateNode("/mydir/oldname");
        node->rename("/mydir2/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir2/oldname"));
    }

    void Test_NestedToNested_NewName() {
        this->CreateDir("/mydir");
        this->CreateDir("/mydir2");
        auto node = this->CreateNode("/mydir/oldname");
        node->rename("/mydir2/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir2/newname"));
    }

    void Test_ToItself() {
        auto node = this->CreateNode("/oldname");
        node->rename("/oldname");
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_Overwrite_InSameDir() {
        auto node = this->CreateNode("/oldname");
        this->CreateNode("/newname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Overwrite_InDifferentDir() {
        this->CreateDir("/parent1");
        this->CreateDir("/parent2");
        auto node = this->CreateNode("/parent1/oldname");
        this->CreateNode("/parent2/newname");
        node->rename("/parent2/newname");
        EXPECT_EQ(boost::none, this->device->Load("/parent1/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/parent2/newname"));
    }

    void Test_Overwrite_DoesntHaveSameEntryTwice() {
        auto node = this->CreateNode("/oldname");
        this->CreateNode("/newname");
        EXPECT_EQ(4u, this->LoadDir("/")->children().size()); // 4, because of '.' and '..'
        node->rename("/newname");
        EXPECT_EQ(3u, this->LoadDir("/")->children().size()); // 3, because of '.' and '..'
    }

    void Test_Overwrite_Error_DirWithFile_InSameDir() {
        this->CreateFile("/oldname");
        this->CreateDir("/newname");
        try {
            this->Load("/oldname")->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EISDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Overwrite_Error_DirWithFile_InDifferentDir() {
        this->CreateDir("/parent1");
        this->CreateDir("/parent2");
        this->CreateFile("/parent1/oldname");
        this->CreateDir("/parent2/newname");
        try {
            this->Load("/parent1/oldname")->rename("/parent2/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EISDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/parent1/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/parent2/newname"));
    }

    void Test_Overwrite_Error_FileWithDir_InSameDir() {
        this->CreateDir("/oldname");
        this->CreateFile("/newname");
        try {
            this->Load("/oldname")->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Overwrite_Error_FileWithDir_InDifferentDir() {
        this->CreateDir("/parent1");
        this->CreateDir("/parent2");
        this->CreateDir("/parent1/oldname");
        this->CreateFile("/parent2/newname");
        try {
            this->Load("/parent1/oldname")->rename("/parent2/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/parent1/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/parent2/newname"));
    }

    void Test_CanRenameTwice() {
        // Test that the node object stays valid after a rename, even if it now points to an entry of a different parent directory.
        this->CreateDir("/mydir1");
        this->CreateDir("/mydir2");
        auto node = this->CreateNode("/oldname");
        node->rename("/mydir1/newname");
        node->rename("/mydir2/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_EQ(boost::none, this->device->Load("/mydir1/newname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir2/newname"));
    }
};

REGISTER_NODE_TEST_SUITE(FsppNodeTest_Rename,
    Error_TargetParentDirDoesntExist,
    Error_TargetParentDirIsFile,
    Error_RootDir,
    InRoot,
    InNested,
    RootToNested_SameName,
    RootToNested_NewName,
    NestedToRoot_SameName,
    NestedToRoot_NewName,
    NestedToNested_SameName,
    NestedToNested_NewName,
    ToItself,
    Overwrite_InSameDir,
    Overwrite_InDifferentDir,
    Overwrite_DoesntHaveSameEntryTwice,
    Overwrite_Error_DirWithFile_InSameDir,
    Overwrite_Error_DirWithFile_InDifferentDir,
    Overwrite_Error_FileWithDir_InSameDir,
    Overwrite_Error_FileWithDir_InDifferentDir,
    CanRenameTwice
);

#endif

//TODO Test for rename (success AND error cases) that stat values stay unchanged (i.e. mode, uid, gid, access times, ...)
//TODO Test for rename (success AND error cases) that contents stay unchanged (i.e. file contents, directory children, symlink target)
//TODO (here and in other fstest operations): Test error paths
