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
    this->CreateSymlink("/mysymlink");
    this->setModificationTimestampLaterThanAccessTimestamp("/mysymlink"); // to make sure that even in relatime behavior, the read access below changes the access timestamp
    auto operation = [this] () {
        this->LoadSymlink("/mysymlink")->target();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mysymlink", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

REGISTER_TYPED_TEST_CASE_P(FsppSymlinkTest_Timestamps,
   target
);

#endif
