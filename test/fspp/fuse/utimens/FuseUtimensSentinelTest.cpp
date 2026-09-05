#include "testutils/FuseUtimensTest.h"
#include <cpp-utils/system/stat.h>
#include <cpp-utils/system/time.h>

#include <array>
#include <fcntl.h>
#include <sys/stat.h>

using ::testing::Eq;
using ::testing::Invoke;
using ::testing::_;

// libfuse 3 hands the filesystem UTIME_NOW / UTIME_OMIT in tv_nsec instead of a concrete
// timestamp (libfuse 2 only did that for filesystems setting flag_utime_omit_ok, which CryFS
// never set). fspp resolves them before calling into the filesystem, so the filesystem must
// never see a sentinel value. These tests drive utimensat(2) through a real mount and check
// what arrives at Filesystem::utimens.

namespace {
constexpr time_t EXISTING_ATIME_SEC = 1500000000;
constexpr long EXISTING_ATIME_NSEC = 123456;
constexpr time_t EXISTING_MTIME_SEC = 1400000000;
constexpr long EXISTING_MTIME_NSEC = 654321;

// A timestamp is "resolved" if it is a real point in time rather than one of the sentinels.
MATCHER(IsNotASentinel, "") {
  return arg.tv_nsec != UTIME_NOW && arg.tv_nsec != UTIME_OMIT;
}

MATCHER_P2(IsCloseToNow, toleranceSeconds, now, "") {
  if (arg.tv_nsec == UTIME_NOW || arg.tv_nsec == UTIME_OMIT) {
    return false;
  }
  const time_t diff = arg.tv_sec > now.tv_sec ? arg.tv_sec - now.tv_sec : now.tv_sec - arg.tv_sec;
  return diff <= toleranceSeconds;
}
}

class FuseUtimensSentinelTest: public FuseUtimensTest {
public:
  // The file already has known timestamps, so we can tell "kept unchanged" apart from "reset".
  void ReturnFileWithKnownTimestampsOnLstat(const char *filename) {
    EXPECT_CALL(*fsimpl, lstat(Eq(filename), _)).WillRepeatedly(Invoke([] (const boost::filesystem::path&, fspp::fuse::STAT *result) {
      result->st_mode = S_IFREG | S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
      result->st_nlink = 1;
      result->st_size = 0;
      result->st_atim.tv_sec = EXISTING_ATIME_SEC;
      result->st_atim.tv_nsec = EXISTING_ATIME_NSEC;
      result->st_mtim.tv_sec = EXISTING_MTIME_SEC;
      result->st_mtim.tv_nsec = EXISTING_MTIME_NSEC;
    }));
  }
};

TEST_F(FuseUtimensSentinelTest, NullTimes_SetsBothToNow) {
  // This is what plain `touch` does.
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec now = cpputils::time::now();
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), IsCloseToNow(60, now), IsCloseToNow(60, now))).Times(1).WillOnce(::testing::Return());

  Utimensat(FILENAME, nullptr);
}

TEST_F(FuseUtimensSentinelTest, BothNow_SetsBothToNow) {
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec now = cpputils::time::now();
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), IsCloseToNow(60, now), IsCloseToNow(60, now))).Times(1).WillOnce(::testing::Return());

  const std::array<struct timespec, 2> times{{{0, UTIME_NOW}, {0, UTIME_NOW}}};
  Utimensat(FILENAME, times.data());
}

TEST_F(FuseUtimensSentinelTest, OmitMtime_KeepsExistingMtime) {
  // `touch -a`: set atime, leave mtime alone. The mtime we pass on must be the one the node has.
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec expectedMtime = makeTimespec(EXISTING_MTIME_SEC, EXISTING_MTIME_NSEC);
  const timespec newAtime = makeTimespec(1234567890, 42);
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), TimeSpecEq(newAtime), TimeSpecEq(expectedMtime))).Times(1).WillOnce(::testing::Return());

  const std::array<struct timespec, 2> times{{newAtime, {0, UTIME_OMIT}}};
  Utimensat(FILENAME, times.data());
}

TEST_F(FuseUtimensSentinelTest, OmitAtime_KeepsExistingAtime) {
  // `touch -m`: set mtime, leave atime alone.
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec expectedAtime = makeTimespec(EXISTING_ATIME_SEC, EXISTING_ATIME_NSEC);
  const timespec newMtime = makeTimespec(1234567890, 42);
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), TimeSpecEq(expectedAtime), TimeSpecEq(newMtime))).Times(1).WillOnce(::testing::Return());

  const std::array<struct timespec, 2> times{{{0, UTIME_OMIT}, newMtime}};
  Utimensat(FILENAME, times.data());
}

TEST_F(FuseUtimensSentinelTest, OmitBoth_KeepsBothTimestamps) {
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec expectedAtime = makeTimespec(EXISTING_ATIME_SEC, EXISTING_ATIME_NSEC);
  const timespec expectedMtime = makeTimespec(EXISTING_MTIME_SEC, EXISTING_MTIME_NSEC);
  // The kernel may elide a fully-omitted update; if it does reach us, both values must be unchanged.
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), TimeSpecEq(expectedAtime), TimeSpecEq(expectedMtime))).Times(::testing::AtMost(1)).WillRepeatedly(::testing::Return());

  const std::array<struct timespec, 2> times{{{0, UTIME_OMIT}, {0, UTIME_OMIT}}};
  Utimensat(FILENAME, times.data());
}

TEST_F(FuseUtimensSentinelTest, NowAndOmit_NeverPassesASentinelDown) {
  // Whatever combination arrives, the filesystem below fspp must only ever see real timestamps.
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), IsNotASentinel(), IsNotASentinel())).Times(1).WillOnce(::testing::Return());

  const std::array<struct timespec, 2> times{{{0, UTIME_NOW}, {0, UTIME_OMIT}}};
  Utimensat(FILENAME, times.data());
}

TEST_F(FuseUtimensSentinelTest, ExplicitTimes_ArePassedThroughUnchanged) {
  ReturnFileWithKnownTimestampsOnLstat(FILENAME);
  const timespec atime = makeTimespec(1000000000, 111);
  const timespec mtime = makeTimespec(1000000001, 222);
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), TimeSpecEq(atime), TimeSpecEq(mtime))).Times(1).WillOnce(::testing::Return());

  const std::array<struct timespec, 2> times{{atime, mtime}};
  Utimensat(FILENAME, times.data());
}
