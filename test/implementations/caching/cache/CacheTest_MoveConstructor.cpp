#include <google/gtest/gtest.h>
#include <memory>
#include "../../../../implementations/caching/cache/Cache.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/CopyableMovableValueType.h"

using namespace blockstore::caching;

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;

//Test that Cache uses a move constructor for Value if possible
class CacheTest_MoveConstructor: public Test {
public:
  CacheTest_MoveConstructor() {
    CopyableMovableValueType::numCopyConstructorCalled = 0;
    cache = make_unique<Cache<MinimalKeyType, CopyableMovableValueType>>();
  }
  unique_ptr<Cache<MinimalKeyType, CopyableMovableValueType>> cache;
};

TEST_F(CacheTest_MoveConstructor, MoveIntoCache) {
  cache->push(MinimalKeyType::create(0), CopyableMovableValueType(2));
  CopyableMovableValueType val = cache->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(0, CopyableMovableValueType::numCopyConstructorCalled);
}

TEST_F(CacheTest_MoveConstructor, CopyIntoCache) {
  CopyableMovableValueType value(2);
  cache->push(MinimalKeyType::create(0), value);
  CopyableMovableValueType val = cache->pop(MinimalKeyType::create(0)).value();
  EXPECT_EQ(1, CopyableMovableValueType::numCopyConstructorCalled);
}
