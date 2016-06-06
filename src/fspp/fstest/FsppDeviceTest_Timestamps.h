#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest_Timestamps: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils {
public:
  void Test_Load_While_Loaded() {
    auto file = this->CreateFile("/myfile");
    auto operation = [this, &file] () {
        this->device->Load("/myfile");
    };
    this->EXPECT_OPERATION_DOESNT_UPDATE_TIMESTAMPS(*file, operation);
  }

  void Test_Load_While_Not_Loaded() {
    struct stat oldStat;
    {
        auto file = this->CreateFile("/myfile");
        oldStat = stat(*file);
        this->ensureNodeTimestampsAreOld(*file);
    }

    this->device->Load("/myfile");

    auto file = this->device->Load("/myfile");

    struct stat newStat = stat(*file.value());
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
