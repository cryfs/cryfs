#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPFILETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppFileTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
    cpputils::unique_ref<fspp::File> CreateFileWithSize(const boost::filesystem::path &path, off_t size) {
        auto file = this->CreateFile(path);
        file->truncate(size);
        cpputils::destruct(std::move(file));
        assert(this->stat(*this->Load(path)).st_size == size);
        return this->LoadFile(path);
    }
};
TYPED_TEST_CASE_P(FsppFileTest_Timestamps);

TYPED_TEST_P(FsppFileTest_Timestamps, open_nomode) {
    this->CreateFile("/myfile");
    auto operation = [this] () {
        this->LoadFile("/myfile")->open(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_rdonly) {
    this->CreateFile("/myfile");
    auto operation = [this] () {
        this->LoadFile("/myfile")->open(O_RDONLY);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_wronly) {
    this->CreateFile("/myfile");
    auto operation = [this] () {
        this->LoadFile("/myfile")->open(O_WRONLY);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, open_rdwr) {
    this->CreateFile("/myfile");
    auto operation = [this] () {
        this->LoadFile("/myfile")->open(O_RDWR);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAnyTimestamps});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_empty) {
    this->CreateFileWithSize("/myfile", 0);
    auto operation = [this] () {
        this->LoadFile("/myfile")->truncate(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_empty_to_nonempty) {
    this->CreateFileWithSize("/myfile", 0);
    auto operation = [this] () {
        this->LoadFile("/myfile")->truncate(10);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_empty) {
    this->CreateFileWithSize("/myfile", 10);
    auto operation = [this] () {
        this->LoadFile("/myfile")->truncate(0);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_shrink) {
    this->CreateFileWithSize("/myfile", 10);
    auto operation = [this] () {
        this->LoadFile("/myfile")->truncate(5);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

TYPED_TEST_P(FsppFileTest_Timestamps, truncate_nonempty_to_nonempty_grow) {
    this->CreateFileWithSize("/myfile", 10);
    auto operation = [this] () {
        this->LoadFile("/myfile")->truncate(20);
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
