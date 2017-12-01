#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_

#include "testutils/FileTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    void IN_STAT(fspp::OpenFile *openFile, std::function<void (struct stat)> callback) {
        struct stat st{};
        openFile->stat(&st);
        callback(st);
    }

    void EXPECT_SIZE(uint64_t expectedSize, fspp::OpenFile *openFile) {
        IN_STAT(openFile, [expectedSize] (struct stat st) {
            EXPECT_EQ(expectedSize, (uint64_t)st.st_size);
        });

        EXPECT_NUMBYTES_READABLE(expectedSize, openFile);
    }

    void EXPECT_NUMBYTES_READABLE(uint64_t expectedSize, fspp::OpenFile *openFile) {
        cpputils::Data data(expectedSize);
        //Try to read one byte more than the expected size
        ssize_t readBytes = openFile->read(data.data(), expectedSize+1, 0);
        //and check that it only read the expected size (but also not less)
        EXPECT_EQ(expectedSize, (uint64_t)readBytes);
    }
};

TYPED_TEST_CASE_P(FsppOpenFileTest);

TYPED_TEST_P(FsppOpenFileTest, CreatedFileIsEmpty) {
    auto file = this->CreateFile("/myfile");
    auto openFile = this->LoadFile("/myfile")->open(O_RDONLY);
    this->EXPECT_SIZE(0, openFile.get());
}

TYPED_TEST_P(FsppOpenFileTest, FileIsFile) {
    auto file = this->CreateFile("/myfile");
    auto openFile = this->LoadFile("/myfile")->open(O_RDONLY);
    this->IN_STAT(openFile.get(), [] (struct stat st) {
        EXPECT_TRUE(S_ISREG(st.st_mode));
    });
}

REGISTER_TYPED_TEST_CASE_P(FsppOpenFileTest,
    CreatedFileIsEmpty,
    FileIsFile
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
