#include "testutils/CacheTest.h"

#include "blockstore/implementations/caching/cache/Cache.h"
#include "testutils/MinimalKeyType.h"
#include "testutils/MinimalValueType.h"
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <boost/chrono.hpp>
#include <boost/optional.hpp>


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

namespace {
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

  // Both halves of this test only hold if we look at the cache inside a window that is 125ms wide
  // on either side. The first entry has to be old enough to be purged, i.e. older than
  // MAX_LIFETIME_SEC (1000ms) against the 1125ms the two sleeps add up to. The second entry has to
  // still be young enough that the cache isn't allowed to purge it, i.e. younger than
  // PURGE_LIFETIME_SEC (500ms) against the 375ms of the second sleep.
  //
  // A sleep has a lower bound but no upper one, so on a loaded machine the second sleep overshoots
  // and the second entry is legitimately purged before we look - which failed this test even though
  // the cache did exactly what it promises. So measure the age we actually reached instead of
  // assuming the sleeps were exact, and retry an attempt whose result the cache was entitled to
  // produce rather than reporting it as a failure. Note "entitled to": an entry older than
  // PURGE_LIFETIME_SEC that is still cached is not a reason to retry, because purging runs every
  // PURGE_INTERVAL rather than the instant an entry becomes old enough. Only an entry that is both
  // old enough to purge and actually gone leaves us without a verdict. The assertions themselves are
  // unchanged: a cache that purges too early, or doesn't purge at all, still fails.
  constexpr int MAX_ATTEMPTS = 5;
  for (int attempt = 1; attempt <= MAX_ATTEMPTS; ++attempt) {
    const bool lastAttempt = (attempt == MAX_ATTEMPTS);

    push(10, 20);
    boost::this_thread::sleep_for(boost::chrono::milliseconds(static_cast<int>(1000 * TIMEOUT1_SEC)));
    const boost::chrono::steady_clock::time_point secondEntryPushedAt = boost::chrono::steady_clock::now();
    push(20, 30);
    boost::this_thread::sleep_for(boost::chrono::milliseconds(static_cast<int>(1000 * TIMEOUT2_SEC)));

    const boost::optional<int> firstEntry = pop(10);
    const boost::optional<int> secondEntry = pop(20);
    // Taken after both pops, so it is an upper bound for the second entry's age at either of them.
    const double secondEntryAgeSec = boost::chrono::duration<double>(
        boost::chrono::steady_clock::now() - secondEntryPushedAt).count();
    const int secondEntryAgeMs = static_cast<int>(1000 * secondEntryAgeSec);

    if (secondEntryAgeSec < Cache::PURGE_LIFETIME_SEC) {
      // Younger than PURGE_LIFETIME_SEC, so the cache is not allowed to have purged it yet.
      ASSERT_TRUE(secondEntry != boost::none)
          << "The entry pushed " << secondEntryAgeMs << "ms ago is gone, but the cache may not purge entries "
          << "younger than PURGE_LIFETIME_SEC (" << (1000 * Cache::PURGE_LIFETIME_SEC) << "ms)";
    } else if (boost::none == secondEntry) {
      // We were too slow AND it is gone. Purging it was allowed at that age, so its absence says
      // nothing about whether the cache is correct, and this attempt gives no verdict.
      if (lastAttempt) {
        FAIL() << "The entry pushed second was purged before we could read it on all " << MAX_ATTEMPTS
               << " attempts. The last one read it " << secondEntryAgeMs << "ms after pushing it, for a sleep of "
               << (1000 * TIMEOUT2_SEC) << "ms, i.e. this machine is too loaded to run this test.";
      }
      continue;
    }
    // Either it was too young to be purged, or it is older than PURGE_LIFETIME_SEC and still here,
    // which is just as correct: purging runs every PURGE_INTERVAL rather than the moment an entry
    // becomes old enough, and entries may live up to MAX_LIFETIME_SEC. Its value has to be right
    // either way.
    EXPECT_EQ(30, secondEntry.value());

    if (boost::none != firstEntry && !lastAttempt) {
      // The first entry is older than MAX_LIFETIME_SEC, so it should be gone - but that needs the
      // purge thread to have been scheduled in time, and from here a purge thread that was starved
      // looks the same as a cache that doesn't purge. Retry. A cache that really never purges fails
      // every attempt and then fails the assertion below on the last one.
      continue;
    }
    EXPECT_EQ(boost::none, firstEntry)
        << "The entry pushed " << (1000 * (TIMEOUT1_SEC + TIMEOUT2_SEC)) << "ms ago is still cached, but entries "
        << "may not get older than MAX_LIFETIME_SEC (" << (1000 * Cache::MAX_LIFETIME_SEC) << "ms)";
    return;
  }
}
