#include <gtest/gtest.h>
#include <blockstore/implementations/caching/IntervalSet.h>
#include "testutils/CallbackMock.h"

using blockstore::caching::IntervalSet;
using testing::Values;
using testing::Test;
using testing::WithParamInterface;
using std::function;
using std::vector;

// Testing an IntervalSet with one covered region: 2->5
class IntervalSetTest_OneRegion : public Test, public WithParamInterface<function<void (IntervalSet<int> *obj)>> {
public:
    IntervalSetTest_OneRegion() {
        GetParam()(&obj);
    }
    IntervalSet<int> obj;

    vector<bool> getCoveredRegion(const IntervalSet<int> &testObj) {
        vector<bool> marker = {false, false, false, false, false, false, false};
        testObj.forEachInterval([&marker] (int begin, int end) {
            for (int i = begin; i < end; ++i) {
                marker[i] = true;
            }
        });
        return marker;
    }
};
INSTANTIATE_TEST_CASE_P(IntervalSetTest_OneRegion, IntervalSetTest_OneRegion, Values(
        [] (IntervalSet<int> *obj) {obj->add(2,5);}, // just one interval
        [] (IntervalSet<int> *obj) {obj->add(3,4); obj->add(2,5);}, // nested intervals 1
        [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(3,4);}, // nested intervals 2
        [] (IntervalSet<int> *obj) {obj->add(2,3); obj->add(4,5); obj->add(3,4);}, // three merged intervals
        [] (IntervalSet<int> *obj) {obj->add(2,2); obj->add(2,5);}, // two intervals, touching at left border
        [] (IntervalSet<int> *obj) {obj->add(2,3); obj->add(3,5);}, // two intervals, touching at left inner
        [] (IntervalSet<int> *obj) {obj->add(2,4); obj->add(4,5);}, // two intervals, touching at right inner
        [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(5,5);}, // two intervals, touching at right border
        [] (IntervalSet<int> *obj) {obj->add(2,4); obj->add(3,5);}, // two intervals, overlapping
        [] (IntervalSet<int> *obj) {obj->add(4,5); obj->add(2,4);} // two intervals, adding intervals in backward order
));

TEST_P(IntervalSetTest_OneRegion, nullregion_leftout) {
    EXPECT_TRUE(obj.isCovered(0,0));
}

TEST_P(IntervalSetTest_OneRegion, nullregion_leftborder) {
    EXPECT_TRUE(obj.isCovered(2,2));
}

TEST_P(IntervalSetTest_OneRegion, nullregion_inner) {
    EXPECT_TRUE(obj.isCovered(3,3));
}

TEST_P(IntervalSetTest_OneRegion, nullregion_rightborder) {
    EXPECT_TRUE(obj.isCovered(5,5));
}

TEST_P(IntervalSetTest_OneRegion, nullregion_rightout) {
    EXPECT_TRUE(obj.isCovered(6,6));
}

TEST_P(IntervalSetTest_OneRegion, leftout_to_leftout) {
    EXPECT_FALSE(obj.isCovered(0,1));
}

TEST_P(IntervalSetTest_OneRegion, leftout_to_leftborder) {
    EXPECT_FALSE(obj.isCovered(1,2));
}

TEST_P(IntervalSetTest_OneRegion, leftout_to_inner) {
    EXPECT_FALSE(obj.isCovered(0,3));
}

TEST_P(IntervalSetTest_OneRegion, leftout_to_rightborder) {
    EXPECT_FALSE(obj.isCovered(0,5));
}

TEST_P(IntervalSetTest_OneRegion, leftout_to_rightout) {
    EXPECT_FALSE(obj.isCovered(0,6));
}

TEST_P(IntervalSetTest_OneRegion, leftborder_to_inner) {
    EXPECT_TRUE(obj.isCovered(2,4));
}

TEST_P(IntervalSetTest_OneRegion, leftborder_to_rightborder) {
    EXPECT_TRUE(obj.isCovered(2,5));
}

TEST_P(IntervalSetTest_OneRegion, leftborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(2,6));
}

TEST_P(IntervalSetTest_OneRegion, inner_to_inner) {
    EXPECT_TRUE(obj.isCovered(3,4));
}

TEST_P(IntervalSetTest_OneRegion, inner_to_rightborder) {
    EXPECT_TRUE(obj.isCovered(3,5));
}

TEST_P(IntervalSetTest_OneRegion, inner_to_rightout) {
    EXPECT_FALSE(obj.isCovered(3,6));
}

TEST_P(IntervalSetTest_OneRegion, rightborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(5,6));
}

TEST_P(IntervalSetTest_OneRegion, forEachInterval) {
    vector<bool> marker = {false, false, false, false, false, false, false};
    obj.forEachInterval([&marker] (int begin, int end) {
        for (int i = begin; i < end; ++i) {
            marker[i] = true;
        }
    });
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false}), marker);
}

TEST_P(IntervalSetTest_OneRegion, IntervalsAreMerged) {
    CallbackMock callback;
    EXPECT_CALL(callback, call(2, 5)).Times(1);
    obj.forEachInterval([&callback] (int begin, int end) {
        callback.call(begin, end);
    });
}

TEST_P(IntervalSetTest_OneRegion, MoveConstructor) {
    IntervalSet<int> target(std::move(obj));
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false}), getCoveredRegion(target));
}

TEST_P(IntervalSetTest_OneRegion, MoveAssignment) {
    IntervalSet<int> target;
    target = std::move(obj);
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false}), getCoveredRegion(target));
}
