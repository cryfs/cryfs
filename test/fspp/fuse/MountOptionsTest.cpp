#include <gtest/gtest.h>

#include "fspp/fuse/MountOptions.h"

using fspp::fuse::forEachMountOption;
using fspp::fuse::removeOptionsRemovedInFuse3;
using std::string;
using std::vector;

namespace {
// Collects every option the walker sees and keeps the ones whose name is in `keep`.
vector<string> walk(vector<string>* fuseOptions, const vector<string>& keep) {
  vector<string> seen;
  forEachMountOption(fuseOptions, [&] (const string& option) {
    seen.push_back(option);
    return keep.end() != std::find(keep.begin(), keep.end(), option);
  });
  return seen;
}
}

TEST(ForEachMountOptionTest, EmptyInput) {
  vector<string> options = {};
  EXPECT_EQ(vector<string>({}), walk(&options, {}));
  EXPECT_EQ(vector<string>({}), options);
}

TEST(ForEachMountOptionTest, OnlyDashOValuesAreVisited) {
  vector<string> options = {"-f", "-o", "allow_other", "--logfile", "mylog"};
  EXPECT_EQ(vector<string>({"allow_other"}), walk(&options, {"allow_other"}));
  EXPECT_EQ(vector<string>({"-f", "-o", "allow_other", "--logfile", "mylog"}), options);
}

TEST(ForEachMountOptionTest, CommaSeparatedValuesAreVisitedOneByOne) {
  vector<string> options = {"-o", "one,two,three"};
  EXPECT_EQ(vector<string>({"one", "two", "three"}), walk(&options, {"one", "two", "three"}));
  EXPECT_EQ(vector<string>({"-o", "one,two,three"}), options);
}

TEST(ForEachMountOptionTest, SeveralDashOArgumentsAreVisited) {
  vector<string> options = {"-o", "one", "-o", "two,three"};
  EXPECT_EQ(vector<string>({"one", "two", "three"}), walk(&options, {"one", "two", "three"}));
  EXPECT_EQ(vector<string>({"-o", "one", "-o", "two,three"}), options);
}

TEST(ForEachMountOptionTest, RemovedOptionIsDroppedFromItsList) {
  vector<string> options = {"-o", "one,two,three"};
  walk(&options, {"one", "three"});
  EXPECT_EQ(vector<string>({"-o", "one,three"}), options);
}

TEST(ForEachMountOptionTest, WhenAllOptionsOfAnArgumentAreRemoved_ThenTheDashOGoesToo) {
  //libfuse rejects a '-o' without a value
  vector<string> options = {"-o", "one,two"};
  walk(&options, {});
  EXPECT_EQ(vector<string>({}), options);
}

TEST(ForEachMountOptionTest, WhenAllOptionsOfOneArgumentAreRemoved_ThenTheOthersSurvive) {
  vector<string> options = {"-f", "-o", "one", "-o", "two", "--logfile", "mylog"};
  walk(&options, {"two"});
  EXPECT_EQ(vector<string>({"-f", "-o", "two", "--logfile", "mylog"}), options);
}

TEST(ForEachMountOptionTest, RemovingTheFirstArgumentDoesntSkipTheNextOne) {
  vector<string> options = {"-o", "one", "-o", "two"};
  EXPECT_EQ(vector<string>({"one", "two"}), walk(&options, {}));
  EXPECT_EQ(vector<string>({}), options);
}

// Every option below was verified against libfuse 3.14.0 and 3.17.2: passing it on makes libfuse
// print "fuse: unknown option(s)" and refuse the mount.
TEST(RemoveOptionsRemovedInFuse3Test, RemovedOptionsAreDropped) {
  for (const char* removed : {"hard_remove", "use_ino", "readdir_ino", "direct_io", "nopath",
                              "intr", "intr_signal=10", "big_writes", "large_read",
                              "max_write=65536"}) {
    vector<string> options = {"-o", removed};
    removeOptionsRemovedInFuse3(&options);
    EXPECT_EQ(vector<string>({}), options) << "'" << removed << "' should have been removed";
  }
}

TEST(RemoveOptionsRemovedInFuse3Test, OptionsLibfuse3StillKnowsAreKept) {
  //these were all verified to mount successfully on libfuse 3.14.0 and 3.17.2
  for (const char* kept : {"allow_other", "allow_root", "default_permissions", "noatime",
                           "kernel_cache", "auto_cache", "umask=022", "uid=0", "gid=0",
                           "entry_timeout=1", "attr_timeout=1"}) {
    vector<string> options = {"-o", kept};
    removeOptionsRemovedInFuse3(&options);
    EXPECT_EQ(vector<string>({"-o", kept}), options) << "'" << kept << "' should have been kept";
  }
}

TEST(RemoveOptionsRemovedInFuse3Test, OnlyTheRemovedOneGoes) {
  vector<string> options = {"-o", "allow_other,big_writes,noatime"};
  removeOptionsRemovedInFuse3(&options);
  EXPECT_EQ(vector<string>({"-o", "allow_other,noatime"}), options);
}

TEST(RemoveOptionsRemovedInFuse3Test, ASimilarlyNamedOptionIsNotTouched) {
  vector<string> options = {"-o", "big_writes_but_not_really,intrepid,max_write_delay=1"};
  removeOptionsRemovedInFuse3(&options);
  EXPECT_EQ(vector<string>({"-o", "big_writes_but_not_really,intrepid,max_write_delay=1"}), options);
}

TEST(RemoveOptionsRemovedInFuse3Test, AnOptionWithoutItsValueIsNotTouched) {
  //'max_write' on its own isn't the removed 'max_write=<n>' option
  vector<string> options = {"-o", "max_write"};
  removeOptionsRemovedInFuse3(&options);
  EXPECT_EQ(vector<string>({"-o", "max_write"}), options);
}

TEST(RemoveOptionsRemovedInFuse3Test, ArgumentsThatArentMountOptionsAreNotTouched) {
  vector<string> options = {"-f", "--logfile", "big_writes"};
  removeOptionsRemovedInFuse3(&options);
  EXPECT_EQ(vector<string>({"-f", "--logfile", "big_writes"}), options);
}
