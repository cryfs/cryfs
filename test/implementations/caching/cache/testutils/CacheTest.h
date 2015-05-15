#pragma once
#ifndef BLOCKS_MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_QUEUEMAPTEST_H_
#define BLOCKS_MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_QUEUEMAPTEST_H_

#include <google/gtest/gtest.h>
#include "../../../../../implementations/caching/cache/Cache.h"
#include "MinimalKeyType.h"
#include "MinimalValueType.h"
#include <boost/optional.hpp>

// This class is a parent class for tests on QueueMap.
// It offers functions to work with a QueueMap test object which is built using types having only the minimal type requirements.
// Furthermore, the class checks that there are no memory leaks left after destructing the QueueMap (by counting leftover instances of Keys/Values).
class CacheTest: public ::testing::Test {
public:
  void push(int key, int value);
  boost::optional<int> pop(int key);

  using Cache = blockstore::caching::Cache<MinimalKeyType, MinimalValueType>;

private:
  Cache _cache;
};

#endif
