#include <gtest/gtest.h>
#include <cpp-utils/system/path.h>

using cpputils::path_is_just_drive_letter;

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
