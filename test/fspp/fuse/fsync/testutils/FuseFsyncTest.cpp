#include "FuseFsyncTest.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFsyncTest::FsyncFile(const char *filename) {
  int error = FsyncFileReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseFsyncTest::FsyncFileReturnError(const char *filename) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);
#if defined(_MSC_VER)
  // Windows doesn't know fsync
  HANDLE file_handle = reinterpret_cast<HANDLE>(_get_osfhandle(fd->fd()));
  if (INVALID_HANDLE_VALUE == file_handle) {
	  throw std::runtime_error("Couldn't get native handle from file descriptor");
  }
  BOOL success = FlushFileBuffers(file_handle);
  if (!success) {
	  throw std::runtime_error("FlushFileBuffer failed with error code " + std::to_string(GetLastError()));
  }
#else
  int retval = ::fsync(fd->fd());
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
#endif
}

unique_ref<OpenFileHandle> FuseFsyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDWR);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
