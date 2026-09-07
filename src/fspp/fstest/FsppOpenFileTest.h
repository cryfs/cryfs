#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_

#include "testutils/FileTest.h"

#include <cstring>

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    void IN_STAT(fspp::OpenFile *openFile, std::function<void (const fspp::OpenFile::stat_info&)> callback) {
        auto st = openFile->stat();
        callback(st);
    }

    void EXPECT_SIZE(fspp::num_bytes_t expectedSize, fspp::OpenFile *openFile) {
        IN_STAT(openFile, [expectedSize] (const fspp::OpenFile::stat_info& st) {
            EXPECT_EQ(expectedSize, st.size);
        });

        EXPECT_NUMBYTES_READABLE(expectedSize, openFile);
    }

    void EXPECT_NUMBYTES_READABLE(fspp::num_bytes_t expectedSize, fspp::OpenFile *openFile) {
        cpputils::Data data(expectedSize.value());
        //Try to read one byte more than the expected size
        const fspp::num_bytes_t readBytes = openFile->read(data.data(), expectedSize+fspp::num_bytes_t(1), fspp::num_bytes_t(0));
        //and check that it only read the expected size (but also not less)
        EXPECT_EQ(expectedSize, readBytes);
    }

    static constexpr const char *CONTENT = "content";
    static constexpr fspp::num_bytes_t CONTENT_SIZE = fspp::num_bytes_t(7);

    cpputils::unique_ref<fspp::OpenFile> CreateFileWithContent(const boost::filesystem::path &path) {
        auto openFile = this->CreateFile(path)->open(fspp::openflags_t::RDWR());
        openFile->write(CONTENT, CONTENT_SIZE, fspp::num_bytes_t(0));
        return openFile;
    }

    void EXPECT_CONTENT_READABLE(fspp::OpenFile *openFile) {
        cpputils::Data data(CONTENT_SIZE.value());
        EXPECT_EQ(CONTENT_SIZE, openFile->read(data.data(), CONTENT_SIZE, fspp::num_bytes_t(0)));
        EXPECT_EQ(0, std::memcmp(CONTENT, data.data(), CONTENT_SIZE.value()));
    }
};
template<class ConcreteFileSystemTestFixture> constexpr const char *FsppOpenFileTest<ConcreteFileSystemTestFixture>::CONTENT;
template<class ConcreteFileSystemTestFixture> constexpr fspp::num_bytes_t FsppOpenFileTest<ConcreteFileSystemTestFixture>::CONTENT_SIZE;

TYPED_TEST_SUITE_P(FsppOpenFileTest);

TYPED_TEST_P(FsppOpenFileTest, CreatedFileIsEmpty) {
    auto file = this->CreateFile("/myfile");
    auto openFile = this->LoadFile("/myfile")->open(fspp::openflags_t::RDONLY());
    this->EXPECT_SIZE(fspp::num_bytes_t(0), openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, FileIsFile) {
    auto file = this->CreateFile("/myfile");
    auto openFile = this->LoadFile("/myfile")->open(fspp::openflags_t::RDONLY());
    this->IN_STAT(openFile.get(), [] (const fspp::OpenFile::stat_info& st) {
        EXPECT_TRUE(st.mode.hasFileFlag());
    });
}

// Renaming a file doesn't invalidate file descriptors that are already open for it,
// so all operations have to keep working on an open file that was moved to a different directory.

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDir_Read) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDir_Write) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    openFile->write(this->CONTENT, this->CONTENT_SIZE, this->CONTENT_SIZE);
    this->EXPECT_SIZE(this->CONTENT_SIZE + this->CONTENT_SIZE, openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDir_Stat) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    this->IN_STAT(openFile.get(), [this] (const fspp::OpenFile::stat_info& st) {
        EXPECT_TRUE(st.mode.hasFileFlag());
        EXPECT_EQ(this->CONTENT_SIZE, st.size);
    });
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDir_Truncate) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    openFile->truncate(fspp::num_bytes_t(3));
    this->EXPECT_SIZE(fspp::num_bytes_t(3), openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDir_Flush) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    openFile->flush();
    openFile->fsync();
    openFile->fdatasync();
    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToSameDir_Read) {
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/myrenamedfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToParentDir_Read) {
    this->CreateDir("/mydir");
    auto openFile = this->CreateFileWithContent("/mydir/myfile");
    this->Load("/mydir/myfile")->rename("/myfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToNestedDir_Read) {
    this->CreateDir("/mydir");
    this->CreateDir("/mydir/mynesteddir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/mynesteddir/myfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDirTwice_Read) {
    this->CreateDir("/mydir");
    this->CreateDir("/myotherdir");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");
    this->Load("/mydir/myfile")->rename("/myotherdir/myfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, RenameToOtherDirOverwritingExistingFile_Read) {
    this->CreateDir("/mydir");
    this->CreateFile("/mydir/myfile");
    auto openFile = this->CreateFileWithContent("/myfile");
    this->Load("/myfile")->rename("/mydir/myfile");

    this->EXPECT_CONTENT_READABLE(openFile.get());
}

REGISTER_TYPED_TEST_SUITE_P(FsppOpenFileTest,
    CreatedFileIsEmpty,
    FileIsFile,
    RenameToOtherDir_Read,
    RenameToOtherDir_Write,
    RenameToOtherDir_Stat,
    RenameToOtherDir_Truncate,
    RenameToOtherDir_Flush,
    RenameToSameDir_Read,
    RenameToParentDir_Read,
    RenameToNestedDir_Read,
    RenameToOtherDirTwice_Read,
    RenameToOtherDirOverwritingExistingFile_Read
);

//TODO Test stat
//TODO Test truncate
//TODO Test read
//TODO Test write
//TODO Test flush
//TODO Test fsync
//TODO Test fdatasync
//TODO Test stat on file that was just created (i.e. the OpenFile instance returned by createAndOpenFile)
//TODO Test all operations do (or don't) affect file timestamps correctly

#endif
