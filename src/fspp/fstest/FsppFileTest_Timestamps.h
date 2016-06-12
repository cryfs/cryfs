#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppFileTest_Timestamps: public FileSystemTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils {
public:
    cpputils::unique_ref<fspp::File> CreateFileWithSize(const boost::filesystem::path &path, off_t size) {
        auto file = this->CreateFile(path);
        file->truncate(size);
        assert(stat(*file).st_size == size);
        return file;
    }
};
TYPED_TEST_CASE_P(FsppFileTest_Timestamps);

TYPED_TEST_P(FsppFileTest_Timestamps, open_nomode) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_rdonly) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(O_RDONLY);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_wronly) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(O_WRONLY);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_rdwr) {
    auto file = this->CreateFile("/myfile");
    auto operation = [&file] () {
        file->open(O_RDWR);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_empty) {
    auto file = this->CreateFileWithSize("/myfile", 0);
    auto operation = [&file] () {
        file->truncate(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_nonempty) {
    auto file = this->CreateFileWithSize("/myfile", 0);
    auto operation = [&file] () {
        file->truncate(10);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_empty) {
    auto file = this->CreateFileWithSize("/myfile", 10);
    auto operation = [&file] () {
        file->truncate(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_shrink) {
    auto file = this->CreateFileWithSize("/myfile", 10);
    auto operation = [&file] () {
        file->truncate(5);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_grow) {
    auto file = this->CreateFileWithSize("/myfile", 10);
    auto operation = [&file] () {
        file->truncate(20);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*file, operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
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
