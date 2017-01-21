#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
};
TYPED_TEST_CASE_P(FsppSymlinkTest_Timestamps);

TYPED_TEST_P(FsppSymlinkTest_Timestamps, target) {
    auto symlink = this->CreateSymlink("/mysymlink");
    auto operation = [&symlink] () {
        symlink->target();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

REGISTER_TYPED_TEST_CASE_P(FsppSymlinkTest_Timestamps,
   target
);

#endif
