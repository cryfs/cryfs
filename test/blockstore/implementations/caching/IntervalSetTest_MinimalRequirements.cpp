#include <gtest/gtest.h>
#include <blockstore/implementations/caching/IntervalSet.h>

using testing::Test;
using blockstore::caching::IntervalSet;

class MinimalIntervalSetType final {
public:
    MinimalIntervalSetType(const MinimalIntervalSetType &rhs) = default; // copyable
    MinimalIntervalSetType &operator=(const MinimalIntervalSetType &rhs) = default; // assignable
private:
    MinimalIntervalSetType() {} // private constructor
    friend class IntervalSetTest_MinimalRequirements;
};

// It is comparable
bool operator<(const MinimalIntervalSetType &, const MinimalIntervalSetType &) {
    return false;
}
bool operator<=(const MinimalIntervalSetType &, const MinimalIntervalSetType &) {
    return true;
}
bool operator==(const MinimalIntervalSetType &, const MinimalIntervalSetType &) {
    return true;
}

// Tests that IntervalSet has minimal requirements on the underlying type
class IntervalSetTest_MinimalRequirements : public Test {
public:
    MinimalIntervalSetType entry;
};

TEST_F(IntervalSetTest_MinimalRequirements, add) {
    IntervalSet<MinimalIntervalSetType> obj;
    obj.add(entry, entry);
}

TEST_F(IntervalSetTest_MinimalRequirements, isCovered) {
    IntervalSet<MinimalIntervalSetType> obj;
    obj.isCovered(entry, entry);
}

TEST_F(IntervalSetTest_MinimalRequirements, forEachInteval) {
    IntervalSet<MinimalIntervalSetType> obj;
    obj.forEachInterval([](MinimalIntervalSetType, MinimalIntervalSetType) {});
}

TEST_F(IntervalSetTest_MinimalRequirements, MoveConstruct) {
    IntervalSet<MinimalIntervalSetType> obj;
    IntervalSet<MinimalIntervalSetType> target(std::move(obj));
}

TEST_F(IntervalSetTest_MinimalRequirements, MoveAssign) {
    IntervalSet<MinimalIntervalSetType> obj;
    IntervalSet<MinimalIntervalSetType> target;
    target = std::move(obj);
}
