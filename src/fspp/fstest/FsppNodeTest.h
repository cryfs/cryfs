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

    void Test_Rename_RootToNested() {
        CreateDir("/mydir");
        auto node = CreateNode("/oldname");
        node->rename("/mydir/newname");
        EXPECT_EQ(boost::none, this->device->Load("/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/mydir/newname"));
    }

    void Test_Rename_NestedToRoot() {
        CreateDir("/mydir");
        auto node = CreateNode("/mydir/oldname");
        node->rename("/newname");
        EXPECT_EQ(boost::none, this->device->Load("/mydir/oldname"));
        EXPECT_NE(boost::none, this->device->Load("/newname"));
    }

    void Test_Rename_NestedToNested() {
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

private:
    void CreateDir(const boost::filesystem::path &path) {
        this->LoadDir(path.parent_path())->createDir(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    }

    void CreateFile(const boost::filesystem::path &path) {
        this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    }
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_File: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().native(), this->MODE_PUBLIC, 0, 0);
        return this->LoadFile(path);
    }
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Dir: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        this->LoadDir(path.parent_path())->createDir(path.filename().native(), this->MODE_PUBLIC, 0, 0);
        return this->LoadDir(path);
    }
};

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Symlink: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {
        this->LoadDir(path.parent_path())->createSymlink(path.filename().native(), "/my/symlink/target", 0, 0);
        return this->LoadSymlink(path);
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
    Rename_RootToNested,
    Rename_NestedToRoot,
    Rename_NestedToNested,
    Rename_ToItself,
    Rename_RootDir
);

#endif
