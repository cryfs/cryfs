#include <google/gtest/gtest.h>
#include <memory>
#include "../../../../implementations/caching/cache/QueueMap.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/CopyableMovableValueType.h"

using namespace blockstore::caching;

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;

//Test that QueueMap uses a move constructor for Value if possible
class QueueMapTest_MoveConstructor: public Test {
public:
  QueueMapTest_MoveConstructor() {
    CopyableMovableValueType::numCopyConstructorCalled = 0;
    map = make_unique<QueueMap<MinimalKeyType, CopyableMovableValueType>>();
  }
  unique_ptr<QueueMap<MinimalKeyType, CopyableMovableValueType>> map;
};

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = map->pop().value();
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_CopyIntoMap) {
  CopyableMovableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = map->pop().value();
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_CopyIntoMap) {
  CopyableMovableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = map->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}
