#include <gtest/gtest.h>
#include <cpp-utils/system/filetime.h>
#include <cpp-utils/tempfile/TempFile.h>

using cpputils::TempFile;
using cpputils::set_filetime;
using cpputils::get_filetime;

TEST(FiletimeTest, SetAndGetTime_ReturnsCorrectTime) {
	TempFile file;
	struct timespec accessTime       { 1535965242, 12345000 };
	struct timespec modificationTime { 1435965242, 98765000 };
	int retval = set_filetime(file.path().string().c_str(), accessTime, modificationTime);
	EXPECT_EQ(0, retval);

	struct timespec readAccessTime{};
	struct timespec readModificationTime{};
	retval = get_filetime(file.path().string().c_str(), &readAccessTime, &readModificationTime);
	EXPECT_EQ(0, retval);

	EXPECT_EQ(accessTime.tv_sec, readAccessTime.tv_sec);
	EXPECT_EQ(modificationTime.tv_sec, readModificationTime.tv_sec);

	// Apple unfortunately doesn't give us nanoseconds at all
#if !defined(__APPLE__)
	EXPECT_EQ(accessTime.tv_nsec, readAccessTime.tv_nsec);
	EXPECT_EQ(modificationTime.tv_nsec, readModificationTime.tv_nsec);
#endif
}
