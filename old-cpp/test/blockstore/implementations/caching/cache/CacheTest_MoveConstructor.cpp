#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "blockstore/implementations/caching/cache/Cache.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/CopyableMovableValueType.h"

using namespace blockstore::caching;

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using ::testing::Test;

//Test that Cache uses a move constructor for Value if possible
class CacheTest_MoveConstructor: public Test {
public:
  CacheTest_MoveConstructor(): cache(make_unique_ref<Cache<MinimalKeyType, CopyableMovableValueType, 100>>("test")) {
    CopyableMovableValueType::numCopyConstructorCalled = 0;
  }
  unique_ref<Cache<MinimalKeyType, CopyableMovableValueType, 100>> cache;
};

TEST_F(CacheTest_MoveConstructor, MoveIntoCache) {
  cache->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = cache->pop(MinimalKeyType::create(0)).value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(CacheTest_MoveConstructor, CopyIntoCache) {
  CopyableMovableValueType value(2);
  cache->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = cache->pop(MinimalKeyType::create(0)).value();
  val.value(); //Access it to avoid the compiler optimizing the assignment away
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}
