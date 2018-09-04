#include "FuseStatfsTest.h"

using std::function;
using ::testing::StrEq;
using ::testing::_;
using ::testing::Invoke;

void FuseStatfsTest::Statfs(const std::string &path) {
  struct ::statvfs dummy{};
  Statfs(path, &dummy);
}

int FuseStatfsTest::StatfsReturnError(const std::string &path) {
  struct ::statvfs dummy{};
  return StatfsReturnError(path, &dummy);
}

void FuseStatfsTest::Statfs(const std::string &path, struct ::statvfs *result) {
  int error = StatfsReturnError(path, result);
  EXPECT_EQ(0, error) << "lstat syscall failed. errno: " << errno;
}

#if defined(_MSC_VER)
#include <Windows.h>
int call_statvfs(const char* filepath, struct ::statvfs * result) {
	ULARGE_INTEGER freeBytesAvailableToCaller, totalNumberOfBytes, totalNumberOfFreeBytes;
	BOOL success = GetDiskFreeSpaceExA(filepath, &freeBytesAvailableToCaller, &totalNumberOfBytes, &totalNumberOfFreeBytes);
	if (!success) {
		throw std::runtime_error("GetDiskFreeSpaceA failed with " + std::to_string(GetLastError()));
	}
	// Hack it together so that DokanY's size calculation does the right thing
	// see https://github.com/dokan-dev/dokany/blob/224fc0880901d86ed98a04e355ee920fe6f095ef/dokan_fuse/src/fusemain.cpp#L947
	// TODO This is likely not going to work because it's not dokan but our own code checking each value individually (e.g. FuseStatfsReturnBfreeTest)
	result->f_bsize = 1;
	result->f_bavail = freeBytesAvailableToCaller.QuadPart;
	result->f_blocks = totalNumberOfBytes.QuadPart;
	result->f_bfree = totalNumberOfFreeBytes.QuadPart;
	return 0;
}
#else
int call_statvfs(const char* filepath, struct :statvfs * result) {
	return ::statvfs(filepath, result);
}
#endif

int FuseStatfsTest::StatfsReturnError(const std::string &path, struct ::statvfs *result) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / path;
  int retval = call_statvfs(realpath.string().c_str(), result);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

struct ::statvfs FuseStatfsTest::CallStatfsWithImpl(function<void(struct ::statvfs*)> implementation) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, statfs(StrEq(FILENAME), _)).WillRepeatedly(Invoke([implementation](const char*, struct ::statvfs *stat) {
    implementation(stat);
  }));

  struct ::statvfs result{};
  Statfs(FILENAME, &result);

  return result;
}
