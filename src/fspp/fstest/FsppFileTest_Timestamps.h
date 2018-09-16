#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppFileTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::File> CreateFileWithSize(const boost::filesystem::path &path, fspp::num_bytes_t size) {
        auto file = this->CreateFile(path);
        file->truncate(size);
        assert(this->stat(*this->Load(path)).size == size);
        return file;
    }
};
TYPED_TEST_CASE_P(FsppFileTest_Timestamps);

TYPED_TEST_P(FsppFileTest_Timestamps, open_nomode) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(fspp::openflags_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_rdonly) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(fspp::openflags_t::RDONLY());
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_wronly) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(fspp::openflags_t::WRONLY());
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}
	
TYPED_TEST_P(FsppFileTest_Timestamps, open_rdwr) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(fspp::openflags_t::RDWR());
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_empty) {
    auto file = this->CreateFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&file] () {
        file->truncate(fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_nonempty) {
    auto file = this->CreateFileWithSize("/myfile", fspp::num_bytes_t(0));
    auto operation = [&file] () {
        file->truncate(fspp::num_bytes_t(10));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_empty) {
    auto file = this->CreateFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&file] () {
        file->truncate(fspp::num_bytes_t(0));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_shrink) {
    auto file = this->CreateFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&file] () {
        file->truncate(fspp::num_bytes_t(5));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_grow) {
    auto file = this->CreateFileWithSize("/myfile", fspp::num_bytes_t(10));
    auto operation = [&file] () {
        file->truncate(fspp::num_bytes_t(20));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

REGISTER_TYPED_TEST_CASE_P(FsppFileTest_Timestamps,
    open_nomode,
    open_rdonly,
    open_wronly,
    open_rdwr,
    truncate_empty_to_empty,
    truncate_empty_to_nonempty,
    truncate_nonempty_to_empty,
    truncate_nonempty_to_nonempty_shrink,
    truncate_nonempty_to_nonempty_grow
);

#endif
