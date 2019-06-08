#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::OpenFile> CreateAndOpenFile(const boost::filesystem::path &path) {
        return this->CreateFile(path)->open(fspp::openflags_t::RDWR());
    }
    cpputils::unique_ref<fspp::OpenFile> CreateAndOpenFileWithSize(const boost::filesystem::path &path, fspp::num_bytes_t size) {
        auto file = this->CreateFile(path);
        file->truncate(size);
        auto openFile = file->open(fspp::openflags_t::RDWR());
        assert(this->stat(*openFile).size == size);
        assert(this->stat(*this->Load(path)).size == size);
        return openFile;
    }
};
TYPED_TEST_CASE_P(FsppOpenFileTest_Timestamps);

TYPED_TEST_P(FsppOpenFileTest_Timestamps, stat) {
    auto openFile = this->CreateAndOpenFile("/mynode");
    auto operation = [&openFile] () {
        openFile->stat();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, truncate_empty_to_empty) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->truncate(fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, truncate_empty_to_nonempty) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->truncate(fspp::num_bytes_t(10));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, truncate_nonempty_to_empty) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&openFile] () {
        openFile->truncate(fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, truncate_nonempty_to_nonempty_shrink) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&openFile] () {
        openFile->truncate(fspp::num_bytes_t(5));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, truncate_nonempty_to_nonempty_grow) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&openFile] () {
        openFile->truncate(fspp::num_bytes_t(20));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, read_inbounds) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&openFile] () {
        std::array<char, 5> buffer{};
        openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, read_outofbounds) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        std::array<char, 5> buffer{};
        openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(2));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, write_inbounds) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&openFile] () {
        openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, write_outofbounds) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(2));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, flush) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->flush();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, fsync) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->fsync();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, fdatasync) {
    auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
    openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
    auto operation = [&openFile] () {
        openFile->fdatasync();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*openFile, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

REGISTER_TYPED_TEST_CASE_P(FsppOpenFileTest_Timestamps,
   stat,
   truncate_empty_to_empty,
   truncate_empty_to_nonempty,
   truncate_nonempty_to_empty,
   truncate_nonempty_to_nonempty_shrink,
   truncate_nonempty_to_nonempty_grow,
   read_inbounds,
   read_outofbounds,
   write_inbounds,
   write_outofbounds,
   flush,
   fsync,
   fdatasync
);

#endif
