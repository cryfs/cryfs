#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_RENAME_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_RENAME_H_

#include "testutils/FsppNodeTest.h"
#include "../fuse/FuseErrnoException.h"

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Rename: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    void Test_TargetParentDirDoesntExist() {
        auto node = this->CreateNode("/oldname");
        try {
            node->rename("/notexistingdir/newname");
            EXPECT_TRUE(false); // Expect it throws an exception
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOENT, e.getErrno());
        }
        //Old file should still exist
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_TargetParentDirIsFile() {
        auto node = this->CreateNode("/oldname");
        this->CreateFile("/somefile");
        try {
            node->rename("/somefile/newname");
            EXPECT_TRUE(false); // Expect it throws an exception
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        //Files should still exist
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/somefile"));
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

    void Test_RootDir() {
        auto root = this->LoadDir("/");
        try {
            root->rename("/newname");
            EXPECT_TRUE(false); // expect throws
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EBUSY, e.getErrno());
        }
    }

    void Test_Overwrite() {
        auto node = this->CreateNode("/oldname");
        this->CreateNode("/newname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Overwrite_DoesntHaveSameEntryTwice() {
        auto node = this->CreateNode("/oldname");
        this->CreateNode("/newname");
        EXPECT_EQ(4u, this->LoadDir("/")->children()->size()); // 4, because of '.' and '..'
        node->rename("/newname");
        EXPECT_EQ(3u, this->LoadDir("/")->children()->size()); // 3, because of '.' and '..'
    }

    void Test_Overwrite_DirWithFile() {
        auto file = this->CreateFile("/oldname");
        this->CreateDir("/newname");
        try {
            file->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EISDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Overwrite_FileWithDir() {
        auto dir = this->CreateDir("/oldname");
        this->CreateFile("/newname");
        try {
            dir->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }
};

REGISTER_NODE_TEST_CASE(FsppNodeTest_Rename,
    TargetParentDirDoesntExist,
    TargetParentDirIsFile,
    InRoot,
    InNested,
    RootToNested_SameName,
    RootToNested_NewName,
    NestedToRoot_SameName,
    NestedToRoot_NewName,
    NestedToNested_SameName,
    NestedToNested_NewName,
    ToItself,
    RootDir,
    Overwrite,
    Overwrite_DoesntHaveSameEntryTwice,
    Overwrite_DirWithFile,
    Overwrite_FileWithDir
);

#endif

//TODO Test for rename (success AND error cases) that stat values stay unchanged (i.e. mode, uid, gid, access times, ...)
//TODO Test for rename (success AND error cases) that contents stay unchanged (i.e. file contents, directory children, symlink target)
//TODO (here and in other fstest operations): Test error paths

//TODO Test all operations do (or don't) affect timestamps correctly