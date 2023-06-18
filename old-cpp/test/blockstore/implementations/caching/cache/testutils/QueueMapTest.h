#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_QUEUEMAPTEST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_CACHING_CACHE_TESTUTILS_QUEUEMAPTEST_H_

#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "blockstore/implementations/caching/cache/QueueMap.h"
#include "MinimalKeyType.h"
#include "MinimalValueType.h"
#include <boost/optional.hpp>

// This class is a parent class for tests on QueueMap.
// It offers functions to work with a QueueMap test object which is built using types having only the minimal type requirements.
// Furthermore, the class checks that there are no memory leaks left after destructing the QueueMap (by counting leftover instances of Keys/Values).
class QueueMapTest: public ::testing::Test {
public:
  QueueMapTest();
  ~QueueMapTest();

  void push(int key, int value);
  boost::optional<int> pop();
  boost::optional<int> pop(int key);
  boost::optional<int> peek();
  int size();

private:
  cpputils::unique_ref<blockstore::caching::QueueMap<MinimalKeyType, MinimalValueType>> _map;
};


#endif
