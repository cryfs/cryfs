#include <gtest/gtest.h>
#include <cpp-utils/system/path.h>
#include <cpp-utils/tempfile/TempDir.h>

using cpputils::TempDir;
using cpputils::path_is_just_drive_letter;
using cpputils::find_longest_existing_path_prefix;

namespace bf = boost::filesystem;

TEST(FindLongestExistingPathPrefixTest, givenEmptyPath_thenReturnsEmptyPath) {
    EXPECT_TRUE(bf::path() == find_longest_existing_path_prefix(bf::path()));
}

TEST(FindLongestExistingPathPrefixTest, givenRootDir_thenReturnsRootDir) {
    EXPECT_TRUE(bf::path("/") == find_longest_existing_path_prefix(bf::path("/")));
}

TEST(FindLongestExistingPathPrefixTest, givenNonexistingTopLevelDir_thenReturnsRootDir) {
    EXPECT_TRUE(bf::path("/") == find_longest_existing_path_prefix(bf::path("/nonexisting_dir")));
}

TEST(FindLongestExistingPathPrefixTest, givenNonexistingTopLevelDirWithSubdir_thenReturnsRootDir) {
    EXPECT_TRUE(bf::path("/") == find_longest_existing_path_prefix(bf::path("/nonexisting_dir/some_subdir")));
}

TEST(FindLongestExistingPathPrefixTest, givenNonexistingNestedDir_thenReturnsExistingPrefix) {
    TempDir dir;
    EXPECT_TRUE(dir.path() == find_longest_existing_path_prefix(dir.path() / "nonexisting_dir"));
}

TEST(FindLongestExistingPathPrefixTest, givenNonexistingNestedDirWithSubdir_thenReturnsExistingPrefix) {
    TempDir dir;
    EXPECT_TRUE(dir.path() == find_longest_existing_path_prefix(dir.path() / "nonexisting_dir" / "some_subdir"));
}

TEST(FindLongestExistingPathPrefixTest, givenExistingNestedDir_thenReturnsDir) {
    TempDir dir;
    EXPECT_TRUE(dir.path() == find_longest_existing_path_prefix(dir.path()));
}

#if defined(_MSC_VER)

TEST(PathTest, pathIsJustDriveLetter) {
    EXPECT_FALSE(path_is_just_drive_letter("C"));
    EXPECT_TRUE(path_is_just_drive_letter("C:"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\"));
    EXPECT_FALSE(path_is_just_drive_letter("C:/"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\test"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\test\\"));
    EXPECT_FALSE(path_is_just_drive_letter("/"));
    EXPECT_FALSE(path_is_just_drive_letter(""));
}

#else

TEST(PathTest, onNonWindowsWeDontHaveDriveLetterPaths) {
    EXPECT_FALSE(path_is_just_drive_letter("C"));
    EXPECT_FALSE(path_is_just_drive_letter("C:"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\"));
    EXPECT_FALSE(path_is_just_drive_letter("C:/"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\test"));
    EXPECT_FALSE(path_is_just_drive_letter("C:\\test\\"));
    EXPECT_FALSE(path_is_just_drive_letter("/"));
    EXPECT_FALSE(path_is_just_drive_letter(""));
}

#endif
