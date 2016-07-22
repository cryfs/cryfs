#include <gtest/gtest.h>
#include <blockstore/implementations/caching/IntervalSet.h>

using blockstore::caching::IntervalSet;
using testing::Values;
using testing::Test;
using testing::WithParamInterface;
using std::function;
using std::vector;

// Testing an IntervalSet with two covered regions: 2->5 and 8->11.
class IntervalSetTest_TwoRegions : public Test, public WithParamInterface<function<void (IntervalSet<int> *obj)>> {
public:
    IntervalSetTest_TwoRegions() {
        GetParam()(&obj);
    }
    IntervalSet<int> obj;

    vector<bool> getCoveredRegion(const IntervalSet<int> &testObj) {
        vector<bool> marker = {false, false, false, false, false, false, false, false, false, false, false, false, false};
        testObj.forEachInterval([&marker] (int begin, int end) {
            for (int i = begin; i < end; ++i) {
                marker[i] = true;
            }
        });
        return marker;
    }
};
INSTANTIATE_TEST_CASE_P(IntervalSetTest_TwoRegions, IntervalSetTest_TwoRegions, Values(
  [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(8,11);}, // adding intervals in forward order
  [] (IntervalSet<int> *obj) {obj->add(8,11); obj->add(2,5);}, // adding intervals in backward order
  [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(50,60); obj->add(8,11);}, // adding third unrelated interval
  [] (IntervalSet<int> *obj) {obj->add(2,3); obj->add(3,4); obj->add(4,5); obj->add(8,11);}, // first region is merged
  [] (IntervalSet<int> *obj) {obj->add(2,4); obj->add(3,5); obj->add(8,11);}, // first region is merged with overlap
  [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(8,9); obj->add(9,10); obj->add(10,11);}, // second region is merged
  [] (IntervalSet<int> *obj) {obj->add(2,5); obj->add(8,10); obj->add(9,11);}, // second region is merged with overlap
  [] (IntervalSet<int> *obj) {obj->add(2,4); obj->add(4,5); obj->add(8,10); obj->add(10,11);}, // both regions are merged
  [] (IntervalSet<int> *obj) {obj->add(2,4); obj->add(3,5); obj->add(8,10); obj->add(9,11);} // both regions are merged with overlap
));

TEST_P(IntervalSetTest_TwoRegions, nullregion_leftout) {
    EXPECT_TRUE(obj.isCovered(0,0));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_firstleftborder) {
    EXPECT_TRUE(obj.isCovered(2,2));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_firstinner) {
    EXPECT_TRUE(obj.isCovered(3,3));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_firstrightborder) {
    EXPECT_TRUE(obj.isCovered(5,5));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_middle) {
    EXPECT_TRUE(obj.isCovered(6,6));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_secondleftborder) {
    EXPECT_TRUE(obj.isCovered(8,8));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_secondinner) {
    EXPECT_TRUE(obj.isCovered(9,9));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_secondrightborder) {
    EXPECT_TRUE(obj.isCovered(11,11));
}

TEST_P(IntervalSetTest_TwoRegions, nullregion_rightout) {
    EXPECT_TRUE(obj.isCovered(12,12));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_leftout) {
    EXPECT_FALSE(obj.isCovered(0,1));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_firstleftborder) {
    EXPECT_FALSE(obj.isCovered(1,2));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_firstinner) {
    EXPECT_FALSE(obj.isCovered(0,3));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_firstrightborder) {
    EXPECT_FALSE(obj.isCovered(0,5));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_middle) {
    EXPECT_FALSE(obj.isCovered(0,6));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_secondleftborder) {
    EXPECT_FALSE(obj.isCovered(1,8));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_secondinner) {
    EXPECT_FALSE(obj.isCovered(0,9));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_secondrightborder) {
    EXPECT_FALSE(obj.isCovered(0,11));
}

TEST_P(IntervalSetTest_TwoRegions, leftout_to_rightout) {
    EXPECT_FALSE(obj.isCovered(0,12));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_firstinner) {
    EXPECT_TRUE(obj.isCovered(2,4));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_firstrightborder) {
    EXPECT_TRUE(obj.isCovered(2,5));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_middle) {
    EXPECT_FALSE(obj.isCovered(2,6));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_secondleftborder) {
    EXPECT_FALSE(obj.isCovered(2,8));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_secondinner) {
    EXPECT_FALSE(obj.isCovered(2,9));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_secondrightborder) {
    EXPECT_FALSE(obj.isCovered(2,11));
}

TEST_P(IntervalSetTest_TwoRegions, firstleftborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(2,12));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_firstinner) {
    EXPECT_TRUE(obj.isCovered(3,4));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_firstrightborder) {
    EXPECT_TRUE(obj.isCovered(3,5));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_middle) {
    EXPECT_FALSE(obj.isCovered(3,6));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_secondleftborder) {
    EXPECT_FALSE(obj.isCovered(3,8));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_secondinner) {
    EXPECT_FALSE(obj.isCovered(3,9));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_secondrightborder) {
    EXPECT_FALSE(obj.isCovered(3,11));
}

TEST_P(IntervalSetTest_TwoRegions, firstinner_to_rightout) {
    EXPECT_FALSE(obj.isCovered(3,12));
}

TEST_P(IntervalSetTest_TwoRegions, firstrightborder_to_middle) {
    EXPECT_FALSE(obj.isCovered(5,6));
}

TEST_P(IntervalSetTest_TwoRegions, firstrightborder_to_secondleftborder) {
    EXPECT_FALSE(obj.isCovered(5,8));
}

TEST_P(IntervalSetTest_TwoRegions, firstrightborder_to_secondinner) {
    EXPECT_FALSE(obj.isCovered(5,9));
}

TEST_P(IntervalSetTest_TwoRegions, firstrightborder_to_secondrightborder) {
    EXPECT_FALSE(obj.isCovered(5,11));
}

TEST_P(IntervalSetTest_TwoRegions, firstrightborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(5,12));
}

TEST_P(IntervalSetTest_TwoRegions, middle_to_middle) {
    EXPECT_FALSE(obj.isCovered(6,7));
}

TEST_P(IntervalSetTest_TwoRegions, middle_to_secondleftborder) {
    EXPECT_FALSE(obj.isCovered(6,8));
}

TEST_P(IntervalSetTest_TwoRegions, middle_to_secondinner) {
    EXPECT_FALSE(obj.isCovered(6,9));
}

TEST_P(IntervalSetTest_TwoRegions, middle_to_secondrightborder) {
    EXPECT_FALSE(obj.isCovered(6,11));
}

TEST_P(IntervalSetTest_TwoRegions, middle_to_rightout) {
    EXPECT_FALSE(obj.isCovered(6,12));
}

TEST_P(IntervalSetTest_TwoRegions, secondleftborder_to_secondinner) {
    EXPECT_TRUE(obj.isCovered(8,9));
}

TEST_P(IntervalSetTest_TwoRegions, secondleftborder_to_secondrightborder) {
    EXPECT_TRUE(obj.isCovered(8,11));
}

TEST_P(IntervalSetTest_TwoRegions, secondleftborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(8,12));
}

TEST_P(IntervalSetTest_TwoRegions, secondinner_to_secondinner) {
    EXPECT_TRUE(obj.isCovered(9,10));
}

TEST_P(IntervalSetTest_TwoRegions, secondinner_to_secondrightborder) {
    EXPECT_TRUE(obj.isCovered(9,11));
}

TEST_P(IntervalSetTest_TwoRegions, secondinner_to_rightout) {
    EXPECT_FALSE(obj.isCovered(9,12));
}

TEST_P(IntervalSetTest_TwoRegions, secondrightborder_to_rightout) {
    EXPECT_FALSE(obj.isCovered(11,12));
}

TEST_P(IntervalSetTest_TwoRegions, rightout_to_rightout) {
    EXPECT_FALSE(obj.isCovered(12,13));
}

TEST_P(IntervalSetTest_TwoRegions, forEachInterval) {
    vector<bool> marker = {false, false, false, false, false, false, false, false, false, false, false, false, false};
    obj.forEachInterval([&marker] (int begin, int end) {
        for (int i = begin; i < end; ++i) {
            marker[i] = true;
        }
    });
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false, false, true, true, true, false, false}), marker);
}

TEST_P(IntervalSetTest_TwoRegions, MoveConstructor) {
    IntervalSet<int> target(std::move(obj));
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false, false, true, true, true, false, false}), getCoveredRegion(target));
}

TEST_P(IntervalSetTest_TwoRegions, MoveAssignment) {
    IntervalSet<int> target;
    target = std::move(obj);
    EXPECT_EQ(vector<bool>({false, false, true, true, true, false, false, false, true, true, true, false, false}), getCoveredRegion(target));
}
