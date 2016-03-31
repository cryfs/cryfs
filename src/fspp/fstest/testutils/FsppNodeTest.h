#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FSPPNODETEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FSPPNODETEST_H_

#include "FileSystemTest.h"
#include <boost/preprocessor/cat.hpp>
#include <boost/preprocessor/variadic/to_seq.hpp>
#include <boost/preprocessor/seq/for_each.hpp>

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    virtual cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) = 0;

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

#define REGISTER_SINGLE_NODE_TEST_CASE(r, Class, Name)                                                                  \
    TYPED_TEST_P(Class, Name) {                                                                                         \
      this->BOOST_PP_CAT(Test_,Name)();                                                                                 \
    }                                                                                                                   \

#define REGISTER_NODE_TEST_CASES_FOR_CLASS(Class, ...)                                                                  \
    TYPED_TEST_CASE_P(Class);                                                                                           \
    BOOST_PP_SEQ_FOR_EACH(REGISTER_SINGLE_NODE_TEST_CASE, Class, BOOST_PP_VARIADIC_TO_SEQ(__VA_ARGS__));                \
    REGISTER_TYPED_TEST_CASE_P(Class, __VA_ARGS__);                                                                     \

#define REGISTER_NODE_TEST_CASE(Class, ...)                                                                             \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_File: public Class<ConcreteFileSystemTestFixture> {                                                   \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateFile(path);                                                                              \
        }                                                                                                               \
    };                                                                                                                  \
                                                                                                                        \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_Dir: public Class<ConcreteFileSystemTestFixture> {                                                    \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateDir(path);                                                                               \
        }                                                                                                               \
    };                                                                                                                  \
                                                                                                                        \
    template<class ConcreteFileSystemTestFixture>                                                                       \
    class Class##_Symlink: public Class<ConcreteFileSystemTestFixture> {                                                \
    public:                                                                                                             \
        cpputils::unique_ref<fspp::Node> CreateNode(const boost::filesystem::path &path) override {                     \
            return this->CreateSymlink(path);                                                                           \
        }                                                                                                               \
    };                                                                                                                  \
                                                                                                                        \
    REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_File, __VA_ARGS__);                                                      \
    REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_Dir, __VA_ARGS__);                                                       \
    REGISTER_NODE_TEST_CASES_FOR_CLASS(Class##_Symlink, __VA_ARGS__);                                                   \

#define INSTANTIATE_NODE_TEST_CASE(FS_NAME, Class, FIXTURE)                                                             \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_File, FIXTURE);                                                      \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_Dir, FIXTURE);                                                       \
    INSTANTIATE_TYPED_TEST_CASE_P(FS_NAME, Class##_Symlink, FIXTURE);                                                   \

#endif
