#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_

#include "testutils/FileTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
    void EXPECT_NUMBYTES_READABLE(fspp::num_bytes_t expectedSize, fspp::OpenFile *openFile) {
        cpputils::Data data(expectedSize.value());
        //Try to read one byte more than the expected size
        const fspp::num_bytes_t readBytes = openFile->read(data.data(), expectedSize+fspp::num_bytes_t(1), fspp::num_bytes_t(0));
        //and check that it only read the expected size (but also not less)
        EXPECT_EQ(expectedSize, readBytes);
    }
};

TYPED_TEST_SUITE_P(FsppOpenFileTest);

TYPED_TEST_P(FsppOpenFileTest, CreatedFileIsEmpty) {
    auto file = this->CreateFile("/myfile");
    auto openFile = this->LoadFile("/myfile")->open(fspp::openflags_t::RDONLY());
    this->EXPECT_NUMBYTES_READABLE(fspp::num_bytes_t(0), openFile.get());
}

REGISTER_TYPED_TEST_SUITE_P(FsppOpenFileTest,
    CreatedFileIsEmpty
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
