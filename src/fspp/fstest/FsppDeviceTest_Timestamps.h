#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
  void Test_Load_While_Loaded() {
    auto node = this->CreateNode("/mynode");
    auto operation = [this] () {
        this->device->Load("/mynode");
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation, {this->ExpectDoesntUpdateAnyTimestamps});
  }

  void Test_Load_While_Not_Loaded() {
    struct stat oldStat{};
    {
        auto node = this->CreateNode("/mynode");
        oldStat = this->stat(*node);
        this->ensureNodeTimestampsAreOld(oldStat);
    }

    this->device->Load("/myfile");

    auto node = this->device->Load("/mynode");

    //Test that timestamps didn't change
    struct stat newStat = this->stat(*node.value());
    EXPECT_EQ(oldStat.st_atim, newStat.st_atim);
    EXPECT_EQ(oldStat.st_mtim, newStat.st_mtim);
    EXPECT_EQ(oldStat.st_ctim, newStat.st_ctim);
  }
};

REGISTER_NODE_TEST_CASE(FsppDeviceTest_Timestamps,
    Load_While_Loaded,
    Load_While_Not_Loaded
);

#endif
