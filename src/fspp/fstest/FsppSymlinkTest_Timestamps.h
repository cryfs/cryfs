#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest_Timestamps: public FileSystemTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils {
public:
};
TYPED_TEST_CASE_P(FsppSymlinkTest_Timestamps);

TYPED_TEST_P(FsppSymlinkTest_Timestamps, target) {
    auto symlink = this->CreateSymlink("/mysymlink");
    auto operation = [&symlink] () {
        symlink->target();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS(*symlink, operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

REGISTER_TYPED_TEST_CASE_P(FsppSymlinkTest_Timestamps,
   target
);

#endif
