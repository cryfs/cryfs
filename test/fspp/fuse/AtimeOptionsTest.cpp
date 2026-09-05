#include <gtest/gtest.h>

#include <fspp/fuse/AtimeOptions.h>

#include <string>
#include <vector>

using fspp::fuse::extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse;
using std::string;
using std::vector;

// CryFS implements the atime behaviour itself, so it picks these options out of the fuse options
// and acts on them. Anything libfuse does not know must additionally be stripped before the rest is
// handed to libfuse, because since libfuse 3.15.0 an unknown mount option aborts the mount.

namespace {
vector<string> extractFrom(vector<string> options, vector<string>* remaining) {
  auto result = extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse(&options);
  *remaining = options;
  return result;
}
}

TEST(AtimeOptionsTest, NoOptions) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({}), extractFrom({}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

TEST(AtimeOptionsTest, UnrelatedOptionsArePassedThrough) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({}), extractFrom({"-o", "allow_other"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "allow_other"}), remaining);
}

// libfuse dropped 'atime' from its mount option table, so forwarding it aborts the mount on
// libfuse >= 3.15. We still have to recognise it, because it selects our own atime behaviour.
TEST(AtimeOptionsTest, Atime_IsRecognisedButNotForwardedToLibfuse) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"atime"}), extractFrom({"-o", "atime"}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

// 'noatime' is still a valid libfuse mount option and maps to a kernel flag, so we pass it on.
TEST(AtimeOptionsTest, Noatime_IsRecognisedAndForwardedToLibfuse) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"noatime"}), extractFrom({"-o", "noatime"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "noatime"}), remaining);
}

TEST(AtimeOptionsTest, Relatime_IsRecognisedButNotForwarded) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"relatime"}), extractFrom({"-o", "relatime"}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

TEST(AtimeOptionsTest, Strictatime_IsRecognisedButNotForwarded) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"strictatime"}), extractFrom({"-o", "strictatime"}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

TEST(AtimeOptionsTest, Nodiratime_IsRecognisedButNotForwarded) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"nodiratime"}), extractFrom({"-o", "nodiratime"}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

TEST(AtimeOptionsTest, CsvConcatenated_OnlyTheForwardableOneRemains) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"atime", "nodiratime"}), extractFrom({"-o", "atime,nodiratime"}, &remaining));
  EXPECT_EQ(vector<string>({}), remaining);
}

TEST(AtimeOptionsTest, CsvConcatenatedWithAnUnrelatedOption_KeepsTheUnrelatedOne) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"atime"}), extractFrom({"-o", "atime,allow_other"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "allow_other"}), remaining);
}

TEST(AtimeOptionsTest, CsvConcatenatedNoatime_IsKeptForLibfuse) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"noatime", "nodiratime"}), extractFrom({"-o", "noatime,nodiratime"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "noatime"}), remaining);
}

TEST(AtimeOptionsTest, SeveralDashOGroups) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({"atime", "nodiratime"}),
            extractFrom({"-o", "atime", "-o", "allow_other", "-o", "nodiratime"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "allow_other"}), remaining);
}

// An option that merely starts with the same letters must not be mistaken for an atime option.
TEST(AtimeOptionsTest, SimilarlyNamedOptionIsNotTouched) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({}), extractFrom({"-o", "atime_is_not_this"}, &remaining));
  EXPECT_EQ(vector<string>({"-o", "atime_is_not_this"}), remaining);
}

// A value that is not preceded by '-o' is not a mount option and must be left alone.
TEST(AtimeOptionsTest, WithoutDashO_NothingIsExtracted) {
  vector<string> remaining;
  EXPECT_EQ(vector<string>({}), extractFrom({"atime"}, &remaining));
  EXPECT_EQ(vector<string>({"atime"}), remaining);
}
