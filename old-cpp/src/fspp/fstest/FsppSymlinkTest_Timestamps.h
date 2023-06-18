#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
};
TYPED_TEST_SUITE_P(FsppSymlinkTest_Timestamps);

TYPED_TEST_P(FsppSymlinkTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_target) {
    auto operation = [this] {
        auto symlink = this->CreateSymlink("/mysymlink");
        this->setAtimeNewerThanMtimeButBeforeYesterday("/mysymlink");
        return [symlink = std::move(symlink)] {
            symlink->target();
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppSymlinkTest_Timestamps, givenAtimeOlderThanMtime_target) {
    auto operation = [this] {
        auto symlink = this->CreateSymlink("/mysymlink");
        this->setAtimeOlderThanMtime("/mysymlink");
        return [symlink = std::move(symlink)] {
            symlink->target();
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

TYPED_TEST_P(FsppSymlinkTest_Timestamps, givenAtimeNewerThanMtime_target) {
    auto operation = [this] {
        auto symlink = this->CreateSymlink("/mysymlink");
        this->setAtimeNewerThanMtime("/mysymlink");
        return [symlink = std::move(symlink)] {
            symlink->target();
        };
    };
    this->testBuilder()
      .withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    });
}

REGISTER_TYPED_TEST_SUITE_P(FsppSymlinkTest_Timestamps,
   givenAtimeNewerThanMtimeButBeforeYesterday_target,
   givenAtimeNewerThanMtime_target,
   givenAtimeOlderThanMtime_target
);

#endif
