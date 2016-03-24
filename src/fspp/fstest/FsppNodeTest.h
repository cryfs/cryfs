#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_H_

#include "testutils/FileSystemTest.h"
#include <boost/preprocessor/cat.hpp>
#include <boost/preprocessor/variadic/to_seq.hpp>
#include <boost/preprocessor/seq/for_each.hpp>
#include "../fuse/FuseErrnoException.h"

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    virtual cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) = 0;

    void Test_Rename_TargetParentDirDoesntExist() {
        auto node = CreateNode("/oldname");
        try {
            node->rename("/notexistingdir/newname");
            EXPECT_TRUE(false); // Expect it throws an exception
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOENT, e.getErrno());
        }
        //Old file should still exist
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_Rename_TargetParentDirIsFile() {
        auto node = CreateNode("/oldname");
        CreateFile("/somefile");
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

    void Test_Rename_InRoot() {
        auto node = CreateNode("/oldname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Rename_InNested() {
        CreateDir("/mydir");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/mydir/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/newname"));
    }

    void Test_Rename_RootToNested_SameName() {
        CreateDir("/mydir");
        auto node = CreateNode("/oldname");
        node->rename("/mydir/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/oldname"));
    }

    void Test_Rename_RootToNested_NewName() {
        CreateDir("/mydir");
        auto node = CreateNode("/oldname");
        node->rename("/mydir/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/newname"));
    }

    void Test_Rename_NestedToRoot_SameName() {
        CreateDir("/mydir");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_Rename_NestedToRoot_NewName() {
        CreateDir("/mydir");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Rename_NestedToNested_SameName() {
        CreateDir("/mydir");
        CreateDir("/mydir2");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/mydir2/oldname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir2/oldname"));
    }

    void Test_Rename_NestedToNested_NewName() {
        CreateDir("/mydir");
        CreateDir("/mydir2");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/mydir2/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir2/newname"));
    }

    void Test_Rename_ToItself() {
        auto node = CreateNode("/oldname");
        node->rename("/oldname");
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
    }

    void Test_Rename_RootDir() {
        auto root = this->LoadDir("/");
        try {
            root->rename("/newname");
            EXPECT_TRUE(false); // expect throws
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EBUSY, e.getErrno());
        }
    }

    void Test_Rename_Overwrite() {
        auto node = CreateNode("/oldname");
        CreateNode("/newname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Rename_Overwrite_DoesntHaveSameEntryTwice() {
        auto node = CreateNode("/oldname");
        CreateNode("/newname");
        EXPECT_EQ(4u, this->LoadDir("/")->children()->size()); // 4, because of '.' and '..'
        node->rename("/newname");
        EXPECT_EQ(3u, this->LoadDir("/")->children()->size()); // 3, because of '.' and '..'
    }

    void Test_Rename_Overwrite_DirWithFile() {
        auto file = CreateFile("/oldname");
        CreateDir("/newname");
        try {
            file->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(EISDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Rename_Overwrite_FileWithDir() {
        auto dir = CreateDir("/oldname");
        CreateFile("/newname");
        try {
            dir->rename("/newname");
            EXPECT_TRUE(false); // expect throw
        } catch (const fspp::fuse::FuseErrnoException &e) {
            EXPECT_EQ(ENOTDIR, e.getErrno());
        }
        EXPECT_NE(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

protected:
    cpputils::unique_ref<fspp::Dir> CreateDir(const boost::filesystem::path &path) {
        this->LoadDir(path.parent_path())->createDir(path.filename().native(), this->MODE_PUBLIC, 0, 0);
        return this->LoadDir(path);
    }

    cpputils::unique_ref<fspp::File> CreateFile(const boost::filesystem::path &path) {
        this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().native(), this->MODE_PUBLIC, 0, 0);
        return this->LoadFile(path);
    }

    cpputils::unique_ref<fspp::Symlink> CreateSymlink(const boost::filesystem::path &path) {
        this->LoadDir(path.parent_path())->createSymlink(path.filename().native(), "/my/symlink/target", 0, 0);
        return this->LoadSymlink(path);
    };
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_File: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        return this->CreateFile(path);
    }
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Dir: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        return this->CreateDir(path);
    }
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Symlink: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        return this->CreateSymlink(path);
    }
};

#define REGISTER_NODE_TEST_CASE(r, Class, Name)                                                                       \
  TYPED_TEST_P(Class, Name) {                                                                                         \
    this->BOOST_PP_CAT(Test_,Name)();                                                                                 \
  }                                                                                                                   \

#define REGISTER_NODE_TEST_CASES_FOR_CLASS(Class, ...)                                                                \
  TYPED_TEST_CASE_P(Class);                                                                                           \
  BOOST_PP_SEQ_FOR_EACH(REGISTER_NODE_TEST_CASE, Class, BOOST_PP_VARIADIC_TO_SEQ(__VA_ARGS__));                       \
  REGISTER_TYPED_TEST_CASE_P(Class, __VA_ARGS__);                                                                     \

#define REGISTER_NODE_TEST_CASES(...)                                                                                 \
  REGISTER_NODE_TEST_CASES_FOR_CLASS(FsppNodeTest_File, __VA_ARGS__);                                                 \
  REGISTER_NODE_TEST_CASES_FOR_CLASS(FsppNodeTest_Dir, __VA_ARGS__);                                                  \
  REGISTER_NODE_TEST_CASES_FOR_CLASS(FsppNodeTest_Symlink, __VA_ARGS__);                                              \

/*
 * Register each of the given test cases for all three classes: FsppNodeTest_File, FsppNodeTest_Dir and FsppNodeTest_Symlink
 */
REGISTER_NODE_TEST_CASES(
    Rename_TargetParentDirDoesntExist,
    Rename_TargetParentDirIsFile,
    Rename_InRoot,
    Rename_InNested,
    Rename_RootToNested_SameName,
    Rename_RootToNested_NewName,
    Rename_NestedToRoot_SameName,
    Rename_NestedToRoot_NewName,
    Rename_NestedToNested_SameName,
    Rename_NestedToNested_NewName,
    Rename_ToItself,
    Rename_RootDir,
    Rename_Overwrite,
    Rename_Overwrite_DoesntHaveSameEntryTwice,
    Rename_Overwrite_DirWithFile,
    Rename_Overwrite_FileWithDir
);

#endif

//TODO Test for rename (success AND error cases) that stat values stay unchanged (i.e. mode, uid, gid, access times, ...)
//TODO Test for rename (success AND error cases) that contents stay unchanged (i.e. file contents, directory children, symlink target)
//TODO (here and in other fstest operations): Test error paths

//TODO Move other applicable test cases from FsppFileTest to here (e.g. utimens, chmod, ...)
//TODO stat
//TODO access
//TODO utimens
//TODO chmod
//TODO chown

//TODO Test all operations do (or don't) affect dir timestamps correctly