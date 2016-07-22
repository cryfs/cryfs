#include <gtest/gtest.h>
#include <blockstore/implementations/caching/IntervalSet.h>

using blockstore::caching::IntervalSet;
using testing::Values;
using testing::Test;
using testing::WithParamInterface;
using std::function;
using std::vector;

class IntervalSetTest_ZeroRegions : public Test, public WithParamInterface<function<void (IntervalSet<int> *obj)>> {
public:
    IntervalSetTest_ZeroRegions() {
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
INSTANTIATE_TEST_CASE_P(IntervalSetTest_ZeroRegions, IntervalSetTest_ZeroRegions, Values(
        [] (IntervalSet<int> */*obj*/) {}, // no regions
        [] (IntervalSet<int> *obj) {obj->add(0,0); obj->add(1,1); obj->add(2,2); obj->add(3,3); obj->add(4,4); obj->add(5,5); obj->add(6,6);} // only empty regions
));

TEST_P(IntervalSetTest_ZeroRegions, nullregion1) {
    EXPECT_TRUE(obj.isCovered(0,0));
}

TEST_P(IntervalSetTest_ZeroRegions, nullregion2) {
    EXPECT_TRUE(obj.isCovered(2,2));
}

TEST_P(IntervalSetTest_ZeroRegions, nullregion3) {
    EXPECT_TRUE(obj.isCovered(-2,-2));
}

TEST_P(IntervalSetTest_ZeroRegions, positiveregion) {
    EXPECT_FALSE(obj.isCovered(1,5));
}

TEST_P(IntervalSetTest_ZeroRegions, regionfromzero) {
    EXPECT_FALSE(obj.isCovered(0,1));
}

TEST_P(IntervalSetTest_ZeroRegions, forEachInterval) {
    vector<bool> marker = {false, false, false, false, false, false, false};
    obj.forEachInterval([&marker] (int begin, int end) {
        for (int i = begin; i < end; ++i) {
            marker[i] = true;
        }
    });
    EXPECT_EQ(vector<bool>({false, false, false, false, false, false, false}), marker);
}

TEST_P(IntervalSetTest_ZeroRegions, MoveConstructor) {
    IntervalSet<int> target(std::move(obj));
    EXPECT_EQ(vector<bool>({false, false, false, false, false, false, false}), getCoveredRegion(target));
}

TEST_P(IntervalSetTest_ZeroRegions, MoveAssignment) {
    IntervalSet<int> target;
    target = std::move(obj);
    EXPECT_EQ(vector<bool>({false, false, false, false, false, false, false}), getCoveredRegion(target));
}
