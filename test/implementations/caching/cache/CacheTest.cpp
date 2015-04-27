#include <google/gtest/gtest.h>

#include "../../../../implementations/caching/cache/Cache.h"

using ::testing::Test;

using namespace blockstore::caching;

class CacheTest: public Test {
public:
  Cache<int, int> cache;
};
