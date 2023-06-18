#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "blockstore/implementations/caching/cache/QueueMap.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/CopyableMovableValueType.h"

using namespace blockstore::caching;

using ::testing::Test;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

//Test that QueueMap uses a move constructor for Value if possible
class QueueMapTest_MoveConstructor: public Test {
public:
  QueueMapTest_MoveConstructor(): map(make_unique_ref<QueueMap<MinimalKeyType, CopyableMovableValueType>>()) {
    CopyableMovableValueType::numCopyConstructorCalled = 0;
  }
  unique_ref<QueueMap<MinimalKeyType, CopyableMovableValueType>> map;
};

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = map->pop().value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_MoveIntoMap) {
  map->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = map->pop(MinimalKeyType::create(0)).value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPopping_CopyIntoMap) {
  CopyableMovableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = map->pop().value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(QueueMapTest_MoveConstructor, PushingAndPoppingPerKey_CopyIntoMap) {
  CopyableMovableValueType value(2);
  map->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = map->pop(MinimalKeyType::create(0)).value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}
