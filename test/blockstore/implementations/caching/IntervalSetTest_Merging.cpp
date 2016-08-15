#include <gtest/gtest.h>
#include <blockstore/implementations/caching/IntervalSet.h>
#include "testutils/CallbackMock.h"

using blockstore::caching::IntervalSet;
using testing::Test;
using std::vector;
using std::pair;

// Testing that IntervalSet merges overlapping intervals
class IntervalSetTest_Merging : public Test {
public:
    IntervalSet<int> obj;

    void EXPECT_HAS_INTERVALS(const vector<pair<int,int>> &expectedIntervals) {
        CallbackMock mock;
        for (const auto &interval : expectedIntervals) {
            EXPECT_CALL(mock, call(interval.first, interval.second)).Times(1);
        }
        obj.forEachInterval([&mock](int begin, int end) { mock.call(begin, end); });
    }
};

TEST_F(IntervalSetTest_Merging, DontMergeSeparate_Forward) {
    obj.add(2, 5);
    obj.add(6, 8);
    EXPECT_HAS_INTERVALS({{2,5}, {6,8}});
}

TEST_F(IntervalSetTest_Merging, DontMergeSeparate_Backward) {
    obj.add(6, 8);
    obj.add(2, 5);
    EXPECT_HAS_INTERVALS({{2,5}, {6,8}});
}

TEST_F(IntervalSetTest_Merging, MergeIdenticalIntervals) {
    obj.add(4, 8);
    obj.add(4, 8);
    EXPECT_HAS_INTERVALS({{4,8}});
}

TEST_F(IntervalSetTest_Merging, MergeNestedIntervals_Forward) {
    obj.add(4, 8);
    obj.add(5, 7);
    EXPECT_HAS_INTERVALS({{4,8}});
}

TEST_F(IntervalSetTest_Merging, MergeNestedIntervals_Backward) {
    obj.add(5, 7);
    obj.add(4, 8);
    EXPECT_HAS_INTERVALS({{4,8}});
}

TEST_F(IntervalSetTest_Merging, MergeTouchingIntervals_Forward) {
    obj.add(2, 5);
    obj.add(5, 8);
    EXPECT_HAS_INTERVALS({{2,8}});
}

TEST_F(IntervalSetTest_Merging, MergeTouchingIntervals_Backward) {
    obj.add(5, 8);
    obj.add(2, 5);
    EXPECT_HAS_INTERVALS({{2,8}});
}

TEST_F(IntervalSetTest_Merging, MergeOverlappingIntervals_Forward) {
    obj.add(2, 5);
    obj.add(4, 8);
    EXPECT_HAS_INTERVALS({{2,8}});
}

TEST_F(IntervalSetTest_Merging, MergeOverlappingIntervals_Backward) {
    obj.add(4, 8);
    obj.add(2, 5);
    EXPECT_HAS_INTERVALS({{2,8}});
}

TEST_F(IntervalSetTest_Merging, MergeThreeIntervals_Touching) {
    obj.add(1, 3);
    obj.add(5, 7);
    obj.add(3, 5);
    EXPECT_HAS_INTERVALS({{1,7}});
}

TEST_F(IntervalSetTest_Merging, MergeThreeIntervals_Overlapping) {
    obj.add(1, 3);
    obj.add(5, 7);
    obj.add(2, 6);
    EXPECT_HAS_INTERVALS({{1,7}});
}

TEST_F(IntervalSetTest_Merging, MergeThreeIntervals_LeftOut) {
    obj.add(1, 3);
    obj.add(5, 7);
    obj.add(0, 6);
    EXPECT_HAS_INTERVALS({{0,7}});
}

TEST_F(IntervalSetTest_Merging, MergeThreeIntervals_RightOut) {
    obj.add(1, 3);
    obj.add(5, 7);
    obj.add(2, 8);
    EXPECT_HAS_INTERVALS({{1,8}});
}

TEST_F(IntervalSetTest_Merging, MergeThreeIntervals_BothOut) {
    obj.add(1, 3);
    obj.add(5, 7);
    obj.add(0, 8);
    EXPECT_HAS_INTERVALS({{0,8}});
}

TEST_F(IntervalSetTest_Merging, MergeFourIntervals_MergeAll_Out) {
    obj.add(2, 3);
    obj.add(5, 7);
    obj.add(8, 9);
    obj.add(0, 10);
    EXPECT_HAS_INTERVALS({{0,10}});
}

TEST_F(IntervalSetTest_Merging, MergeFourIntervals_MergeAll_NotOut) {
    obj.add(2, 3);
    obj.add(5, 7);
    obj.add(8, 9);
    obj.add(3, 8);
    EXPECT_HAS_INTERVALS({{2,9}});
}

TEST_F(IntervalSetTest_Merging, MergeFourIntervals_DontMergeAll) {
    obj.add(0, 1);
    obj.add(2, 3);
    obj.add(5, 7);
    obj.add(8, 9);
    obj.add(10, 11);
    obj.add(3, 8);
    EXPECT_HAS_INTERVALS({{0,1}, {2,9}, {10,11}});
}
