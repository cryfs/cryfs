#include "testutils/CacheTest.h"

#include "blockstore/implementations/caching/cache/Cache.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/MinimalValueType.h"
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>


using namespace blockstore::caching;

class CacheTest_PushAndPop: public CacheTest {};

TEST_F(CacheTest_PushAndPop, PopNonExistingEntry_EmptyCache) {
  EXPECT_EQ(boost::none, pop(10));
}

TEST_F(CacheTest_PushAndPop, PopNonExistingEntry_NonEmptyCache) {
  push(9, 10);
  EXPECT_EQ(boost::none, pop(10));
}

TEST_F(CacheTest_PushAndPop, PopNonExistingEntry_FullCache) {
  //Add a lot of even numbered keys
  for (int i = 0; i < static_cast<int>(MAX_ENTRIES); ++i) {
    push(2*i, 2*i);
  }
  //Request an odd numbered key
  EXPECT_EQ(boost::none, pop(9));
}

TEST_F(CacheTest_PushAndPop, OneEntry) {
  push(10, 20);
  EXPECT_EQ(20, pop(10).value());
}

TEST_F(CacheTest_PushAndPop, MultipleEntries) {
  push(10, 20);
  push(20, 30);
  push(30, 40);
  EXPECT_EQ(30, pop(20).value());
  EXPECT_EQ(20, pop(10).value());
  EXPECT_EQ(40, pop(30).value());
}

TEST_F(CacheTest_PushAndPop, FullCache) {
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); ++i) {
    push(i, 2*i);
  }
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); ++i) {
    EXPECT_EQ(2*i, pop(i).value());
  }
}

TEST_F(CacheTest_PushAndPop, FullCache_PushNonOrdered_PopOrdered) {
  for(int i = 1; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    push(i, 2*i);
  }
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    push(i, 2*i);
  }
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); ++i) {
    EXPECT_EQ(2*i, pop(i).value());
  }
}

TEST_F(CacheTest_PushAndPop, FullCache_PushOrdered_PopNonOrdered) {
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); ++i) {
    push(i, 2*i);
  }
  for(int i = 1; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    EXPECT_EQ(2*i, pop(i).value());
  }
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    EXPECT_EQ(2*i, pop(i).value());
  }
}

int roundDownToEven(int number) {
  if (number % 2 == 0) {
    return number;
  } else {
    return number - 1;
  }
}

int roundDownToOdd(int number) {
  if (number % 2 != 0) {
    return number;
  } else {
    return number - 1;
  }
}

TEST_F(CacheTest_PushAndPop, FullCache_PushNonOrdered_PopNonOrdered) {
  for(int i = roundDownToEven(MAX_ENTRIES - 1); i >= 0; i -= 2) {
    push(i, 2*i);
  }
  for(int i = 1; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    push(i, 2*i);
  }
  for(int i = roundDownToOdd(MAX_ENTRIES-1); i >= 0; i -= 2) {
    EXPECT_EQ(2*i, pop(i).value());
  }
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES); i += 2) {
    EXPECT_EQ(2*i, pop(i).value());
  }
}

TEST_F(CacheTest_PushAndPop, MoreThanFullCache) {
  for(int i = 0; i < static_cast<int>(MAX_ENTRIES + 2); ++i) {
    push(i, 2*i);
  }
  //Check that the oldest two elements got deleted automatically
  EXPECT_EQ(boost::none, pop(0));
  EXPECT_EQ(boost::none, pop(1));
  //Check the other elements are still there
  for(int i = 2; i < static_cast<int>(MAX_ENTRIES + 2); ++i) {
    EXPECT_EQ(2*i, pop(i).value());
  }
}

TEST_F(CacheTest_PushAndPop, AfterTimeout) {
  constexpr double TIMEOUT1_SEC = Cache::MAX_LIFETIME_SEC * 3/4;
  constexpr double TIMEOUT2_SEC = Cache::PURGE_LIFETIME_SEC * 3/4;
  static_assert(TIMEOUT1_SEC + TIMEOUT2_SEC > Cache::MAX_LIFETIME_SEC, "Ensure that our chosen timeouts push the first entry out of the cache");

  push(10, 20);
  boost::this_thread::sleep_for(boost::chrono::milliseconds(static_cast<int>(1000 * TIMEOUT1_SEC)));
  push(20, 30);
  boost::this_thread::sleep_for(boost::chrono::milliseconds(static_cast<int>(1000 * TIMEOUT2_SEC)));
  EXPECT_EQ(boost::none, pop(10));
  EXPECT_EQ(30, pop(20).value());
}
