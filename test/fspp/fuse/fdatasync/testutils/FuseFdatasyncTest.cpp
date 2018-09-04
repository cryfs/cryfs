#include "FuseFdatasyncTest.h"
#include <fcntl.h>
#if defined(_MSC_VER)
#include <io.h>
#endif

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFdatasyncTest::FdatasyncFile(const char *filename) {
  int error = FdatasyncFileReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseFdatasyncTest::FdatasyncFileReturnError(const char *filename) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);

#if defined(_MSC_VER)
  // Windows doesn't know fdatasync
  HANDLE file_handle = reinterpret_cast<HANDLE>(_get_osfhandle(fd->fd()));
  if (INVALID_HANDLE_VALUE == file_handle) {
	  throw std::runtime_error("Couldn't get native handle from file descriptor");
  }
  BOOL success = FlushFileBuffers(file_handle);
  if (!success) {
	  throw std::runtime_error("FlushFileBuffer failed with error code " + std::to_string(GetLastError()));
  }
#elif defined(F_FULLFSYNC)
  // This is MacOSX, which doesn't know fdatasync
  int retval = fcntl(fd->fd(), F_FULLFSYNC);
  if (retval != -1) {
	  return 0;
  }
  else {
	  return errno;
  }
#else
  int retval = ::fdatasync(fd->fd());
  if (retval != -1) {
    return 0;
  } else {
    return errno;
  }
#endif
}

unique_ref<OpenFileHandle> FuseFdatasyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDWR);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
