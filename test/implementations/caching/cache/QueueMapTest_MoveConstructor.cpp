#include <google/gtest/gtest.h>
#include <memory>
#include "../../../../implementations/caching/cache/QueueMap.h"
#include "testutils/MinimalKeyType.h"

using namespace blockstore::caching;

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;

class CopyableValueType {
public:
  static int numCopyConstructorCalled;
  CopyableValueType(int value): _value(value) {}
  CopyableValueType(const CopyableValueType &rhs): CopyableValueType(rhs._value) {
    ++numCopyConstructorCalled;
  }
  CopyableValueType(CopyableValueType &&rhs): CopyableValueType(rhs._value) {
    //Don't increase numCopyConstructorCalled
  }
  int value() const {
    return _value;
  }
private:
  int _value;
};
int CopyableValueType::numCopyConstructorCalled = 0;

//Test that QueueMap uses a move constructor for Value if possible
class QueueMapTest_MoveConstructor: public Test {
public:
  QueueMapTest_MoveConstructor() {
    CopyableValueType::numCopyConstructorCalled = 0;
    map = make_unique<QueueMap<MinimalKeyType, CopyableValueType>>();
  }
  unique_ptr<QueueMap<MinimalKeyType, CopyableValueType>> map;
};

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableValueType(2));
  CopyableValueType val = map->pop().value();
  EXPECT_EQ(0, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableValueType(2));
  CopyableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(0, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_CopyIntoMap) {
  CopyableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableValueType val = map->pop().value();
  EXPECT_EQ(1, CopyableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_CopyIntoMap) {
  CopyableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(1, CopyableValueType::numCopyConstructorCalled);
}
