#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
  void Test_Load_While_Loaded() {
    auto operation = [this] {
        auto node = this->CreateNode("/mynode");
        return [this] {
            this->device->Load("/mynode");
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mynode", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
  }

  void Test_Load_While_Not_Loaded() {
    this->testBuilder().withAnyAtimeConfig([&] {
        fspp::Node::stat_info oldStat{};
        {
            auto node = this->CreateNode("/mynode");
            oldStat = this->stat(*node);
            this->ensureNodeTimestampsAreOld(oldStat);
        }

        this->device->Load("/myfile");

        auto node = this->device->Load("/mynode");

        //Test that timestamps didn't change
        fspp::Node::stat_info newStat = this->stat(*node.value());
        EXPECT_EQ(oldStat.atime, newStat.atime);
        EXPECT_EQ(oldStat.mtime, newStat.mtime);
        EXPECT_EQ(oldStat.ctime, newStat.ctime);
    });
  }
};

REGISTER_NODE_TEST_SUITE(FsppDeviceTest_Timestamps,
    Load_While_Loaded,
    Load_While_Not_Loaded
);

#endif
