#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FSPPNODETEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FSPPNODETEST_H_

#include "FileSystemTest.h"
#include <boost/preprocessor/cat.hpp>
#include <boost/preprocessor/variadic/to_seq.hpp>
#include <boost/preprocessor/seq/for_each.hpp>

class FsppNodeTestBase {
public:
    virtual void IN_STAT(fspp::Node *node, std::function<void (struct stat)> callback) = 0;
    virtual void EXPECT_SIZE(uint64_t expectedSize, fspp::Node *node) = 0;
};

/**
 * Inherit your fixture from this class to write a test case that is run on nodes, i.e. files, directories and symlinks.
 * You can use this->CreateNode() to create a node and then call fspp::Node functions on it.
 * Add your test cases as void Test_xxx() functions to your fixture and register/instantiate them using
 * REGISTER_NODE_TEST_CASE and INSTANTIATE_NODE_TEST_CASE.
 * It will then automatically create a test case for each node type (file, directory, symlink).
 * See FsppNodeTest_Rename for an example.
 */
template<class ConcreteFileSystemTestFixture>
class FsppNodeTest: public virtual FsppNodeTestBase, public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    virtual cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) = 0;
};

class FsppNodeTest_File_Helper: public virtual FsppNodeTestBase {
public:
    void IN_STAT(fspp::Node *file, std::function<void (struct stat)> callback) override {
        struct stat st1, st2;
        file->stat(&st1);
        callback(st1);
        dynamic_cast<fspp::File &>(*file).open(O_RDONLY)->stat(&st2);
        callback(st2);
    }

    void EXPECT_SIZE(uint64_t expectedSize, fspp::Node *node) override {
        IN_STAT(node, [expectedSize] (struct stat st) {
            EXPECT_EQ(expectedSize, (uint64_t)st.st_size);
        });

        EXPECT_NUMBYTES_READABLE(expectedSize, dynamic_cast<fspp::File*>(node));
    }

    void EXPECT_NUMBYTES_READABLE(uint64_t expectedSize, fspp::File *file) {
        auto openFile = file->open(O_RDONLY);
        cpputils::Data data(expectedSize);
        //Try to read one byte more than the expected size
        ssize_t readBytes = openFile->read(data.data(), expectedSize+1, 0);
        //and check that it only read the expected size (but also not less)
        EXPECT_EQ(expectedSize, (uint64_t)readBytes);
    }
};

class FsppNodeTest_Dir_Helper: public virtual FsppNodeTestBase {
public:
    void IN_STAT(fspp::Node *file, std::function<void (struct stat)> callback) override {
        struct stat st;
        file->stat(&st);
        callback(st);
    }

    void EXPECT_SIZE(uint64_t expectedSize, fspp::Node *node) override {
        IN_STAT(node, [expectedSize] (struct stat st) {
            EXPECT_EQ(expectedSize, (uint64_t)st.st_size);
        });
    }
};

class FsppNodeTest_Symlink_Helper: public virtual FsppNodeTestBase {
public:
    void IN_STAT(fspp::Node *file, std::function<void (struct stat)> callback) override {
        struct stat st;
        file->stat(&st);
        callback(st);
    }

    void EXPECT_SIZE(uint64_t expectedSize, fspp::Node *node) override {
        IN_STAT(node, [expectedSize] (struct stat st) {
            EXPECT_EQ(expectedSize, (uint64_t)st.st_size);
        });
    }
};

#define _REGISTER_SINGLE_NODE_TEST_CASE(r, Class, Name)                                                                 \
    TYPED_TEST_P(Class, Name) {                                                                                         \
      this->BOOST_PP_CAT(Test_,Name)();                                                                                 \
    }                                                                                                                   \

#define _REGISTER_NODE_TEST_CASES_FOR_CLASS(Class, ...)                                                                 \
    BOOST_PP_SEQ_FOR_EACH(_REGISTER_SINGLE_NODE_TEST_CASE, Class, BOOST_PP_VARIADIC_TO_SEQ(__VA_ARGS__));               \
    REGISTER_TYPED_TEST_CASE_P(Class, __VA_ARGS__);                                                                     \

#define _REGISTER_FILE_TEST_CASE(Class, ...)                                                                            \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_FileNode: public Class<ConcreteFileSystemTestFixture>, public FsppNodeTest_File_Helper {              \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateFile(path);                                                                              \
        }                                                                                                               \
    };                                                                                                                  \
    TYPED_TEST_CASE_P(Class##_FileNode);                                                                                \
    _REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_FileNode, __VA_ARGS__);                                                 \

#define _REGISTER_DIR_TEST_CASE(Class, ...)                                                                             \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_DirNode: public Class<ConcreteFileSystemTestFixture>, public FsppNodeTest_Dir_Helper {                \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateDir(path);                                                                               \
        }                                                                                                               \
    };                                                                                                                  \
    TYPED_TEST_CASE_P(Class##_DirNode);                                                                                 \
    _REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_DirNode, __VA_ARGS__);                                                  \

#define _REGISTER_SYMLINK_TEST_CASE(Class, ...)                                                                         \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_SymlinkNode: public Class<ConcreteFileSystemTestFixture>, public FsppNodeTest_Symlink_Helper {        \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateSymlink(path);                                                                           \
        }                                                                                                               \
    };                                                                                                                  \
    TYPED_TEST_CASE_P(Class##_SymlinkNode);                                                                             \
    _REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_SymlinkNode, __VA_ARGS__);                                              \

#define REGISTER_NODE_TEST_CASE(Class, ...)                                                                             \
    _REGISTER_FILE_TEST_CASE(Class, __VA_ARGS__);                                                                       \
    _REGISTER_DIR_TEST_CASE(Class, __VA_ARGS__);                                                                        \
    _REGISTER_SYMLINK_TEST_CASE(Class, __VA_ARGS__);                                                                    \

#define INSTANTIATE_NODE_TEST_CASE(FS_NAME, Class, FIXTURE)                                                             \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_FileNode, FIXTURE);                                                  \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_DirNode, FIXTURE);                                                   \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_SymlinkNode, FIXTURE);                                               \

#endif
