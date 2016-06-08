#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPNODETEST_STAT_H_
#define MESSMER_FSPP_FSTEST_FSPPNODETEST_STAT_H_

#include "testutils/FsppNodeTest.h"
#include "../fuse/FuseErrnoException.h"

template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Stat: public FsppNodeTest<ConcreteFileSystemTestFixture> {
public:
    void Test_Nlink() {
        auto node = this->CreateNode("/mynode");
        this->IN_STAT(node.get(), [] (struct stat st) {
            EXPECT_EQ(1u, st.st_nlink);
        });
    }
};

// Test cases only run for file nodes
template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Stat_FileOnly: public FileSystemTest<ConcreteFileSystemTestFixture>, public FsppNodeTest_File_Helper {};

TYPED_TEST_CASE_P(FsppNodeTest_Stat_FileOnly);

TYPED_TEST_P(FsppNodeTest_Stat_FileOnly, CreatedFileIsEmpty) {
    auto file = this->CreateFile("/myfile");
    this->EXPECT_SIZE(0, file.get());
}

TYPED_TEST_P(FsppNodeTest_Stat_FileOnly, FileIsFile) {
    auto file = this->CreateFile("/myfile");
    this->IN_STAT(file.get(), [] (struct stat st) {
        EXPECT_TRUE(S_ISREG(st.st_mode));
    });
}

// Test cases only run for dir nodes
template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Stat_DirOnly: public FileSystemTest<ConcreteFileSystemTestFixture>, public FsppNodeTest_Dir_Helper {};

TYPED_TEST_CASE_P(FsppNodeTest_Stat_DirOnly);

TYPED_TEST_P(FsppNodeTest_Stat_DirOnly, DirIsDir) {
    auto file = this->CreateDir("/mydir");
    this->IN_STAT(file.get(), [] (struct stat st) {
        EXPECT_TRUE(S_ISDIR(st.st_mode));
    });
}

// Test cases only run for symlink nodes
template<class ConcreteFileSystemTestFixture>
class FsppNodeTest_Stat_SymlinkOnly: public FileSystemTest<ConcreteFileSystemTestFixture>, public FsppNodeTest_Symlink_Helper {};

TYPED_TEST_CASE_P(FsppNodeTest_Stat_SymlinkOnly);

TYPED_TEST_P(FsppNodeTest_Stat_SymlinkOnly, SymlinkIsSymlink) {
    auto file = this->CreateSymlink("/mysymlink");
    this->IN_STAT(file.get(), [] (struct stat st) {
        EXPECT_TRUE(S_ISLNK(st.st_mode));
    });
}

REGISTER_NODE_TEST_CASE(FsppNodeTest_Stat,
    Nlink
);

REGISTER_TYPED_TEST_CASE_P(FsppNodeTest_Stat_FileOnly,
    CreatedFileIsEmpty,
    FileIsFile
);

REGISTER_TYPED_TEST_CASE_P(FsppNodeTest_Stat_DirOnly,
    DirIsDir
);

REGISTER_TYPED_TEST_CASE_P(FsppNodeTest_Stat_SymlinkOnly,
    SymlinkIsSymlink
);

#endif

//TODO More test cases
