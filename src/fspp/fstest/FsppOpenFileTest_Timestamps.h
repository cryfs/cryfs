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
        ASSERT(this->stat(*this->Load(path)).size == size, "");
        return openFile;
    }
    void CreateFileWithSize(const boost::filesystem::path &path, fspp::num_bytes_t size) {
        auto file = this->CreateFile(path);
        file->truncate(size);
    }
    cpputils::unique_ref<fspp::OpenFile> OpenFile(const boost::filesystem::path &path, fspp::num_bytes_t size) {
        auto file = this->LoadFile(path);
        auto openFile = file->open(fspp::openflags_t::RDWR());
        ASSERT(this->stat(*this->Load(path)).size == size, "");
        return openFile;
    }
};
TYPED_TEST_SUITE_P(FsppOpenFileTest_Timestamps);

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_read_inbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeNewerThanMtimeButBeforeYesterday(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(0));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeNewerThanMtime_read_inbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeNewerThanMtime(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(0));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeOlderThanMtime_read_inbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeOlderThanMtime(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(0));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_read_outofbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeNewerThanMtimeButBeforeYesterday(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(2));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeNewerThanMtime_read_outofbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeNewerThanMtime(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(2));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, givenAtimeOlderThanMtime_read_outofbounds) {
    const boost::filesystem::path path = "/myfile";
    auto operation = [this, path] () {
        this->CreateFileWithSize(path, fspp::num_bytes_t(10));
        this->setAtimeOlderThanMtime(path);
        auto openFile = this->OpenFile(path, fspp::num_bytes_t(10));

        return [openFile = std::move(openFile)] {
            std::array<char, 5> buffer{};
            openFile->read(buffer.data(), fspp::num_bytes_t(5), fspp::num_bytes_t(2));
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(path, operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, write_inbounds) {
    auto operation = [] (fspp::OpenFile* openFile){
        return [openFile] {
            openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation(openFile.get()), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, write_outofbounds) {
    auto operation = [] (fspp::OpenFile* openFile){
        return [openFile] {
            openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(2));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(0));
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation(openFile.get()), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, flush) {
    auto operation = [] (fspp::OpenFile* openFile){
        openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
        return [openFile] {
            openFile->flush();
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation(openFile.get()), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, fsync) {
    auto operation = [] (fspp::OpenFile* openFile){
        openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
        return [openFile] {
            openFile->fsync();
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation(openFile.get()), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppOpenFileTest_Timestamps, fdatasync) {
    auto operation = [] (fspp::OpenFile* openFile){
        openFile->write("content", fspp::num_bytes_t(7), fspp::num_bytes_t(0));
        return [openFile] {
            openFile->fdatasync();
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        auto openFile = this->CreateAndOpenFileWithSize("/myfile", fspp::num_bytes_t(10));
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/myfile", operation(openFile.get()), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

REGISTER_TYPED_TEST_SUITE_P(FsppOpenFileTest_Timestamps,
   givenAtimeNewerThanMtimeButBeforeYesterday_read_inbounds,
   givenAtimeNewerThanMtime_read_inbounds,
   givenAtimeOlderThanMtime_read_inbounds,
   givenAtimeNewerThanMtimeButBeforeYesterday_read_outofbounds,
   givenAtimeNewerThanMtime_read_outofbounds,
   givenAtimeOlderThanMtime_read_outofbounds,
   write_inbounds,
   write_outofbounds,
   flush,
   fsync,
   fdatasync
);

#endif
