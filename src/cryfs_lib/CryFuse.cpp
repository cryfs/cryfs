#include "CryFuse.h"

#include <sys/types.h>
#include <sys/time.h>
#include <dirent.h>
#include <cassert>

#define UNUSED(expr) (void)(expr)

using fusepp::path;

namespace cryfs {

namespace {
  int errcode_map(int exit_status) {
    if (exit_status < 0) {
      return -errno;
    }
    return 0;
  }
}

CryFuse::CryFuse(CryDevice *device)
  :_device(device) {
}

int CryFuse::getattr(const path &path, struct stat *stbuf) {
  UNUSED(stbuf);
  printf("getattr(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int retstat = lstat(real_path.c_str(), stbuf);
  return errcode_map(retstat);
}

int CryFuse::fgetattr(const path &path, struct stat *stbuf, fuse_file_info *fileinfo) {
  printf("fgetattr(%s, _, _)\n", path.c_str());

  // On FreeBSD, trying to do anything with the mountpoint ends up
  // opening it, and then using the FD for an fgetattr.  So in the
  // special case of a path of "/", I need to do a getattr on the
  // underlying root directory instead of doing the fgetattr().
  if (path.native() == "/") {
    return getattr(path, stbuf);
  }

  int retstat = fstat(fileinfo->fh, stbuf);
  return errcode_map(retstat);
}

int CryFuse::readlink(const path &path, char *buf, size_t size) {
  printf("readlink(%s, _, %zu)\n", path.c_str(), size);
  auto real_path = _device->RootDir() / path;
  //size-1, because the fuse readlink() function includes the null terminating byte in the buffer size,
  //but the posix version does not and also doesn't append one.
  int real_size = ::readlink(real_path.c_str(), buf, size-1);
  if (real_size < 0) {
    return -errno;
  }
  //Terminate the string
  buf[real_size] = '\0';

  return 0;
}

int CryFuse::mknod(const path &path, mode_t mode, dev_t rdev) {
  UNUSED(rdev);
  printf("Called non-implemented mknod(%s, %d, _)\n", path.c_str(), mode);
  return 0;
}

int CryFuse::mkdir(const path &path, mode_t mode) {
  printf("mkdir(%s, %d)\n", path.c_str(), mode);
  auto real_path = _device->RootDir() / path;
  int retstat = ::mkdir(real_path.c_str(), mode);
  return errcode_map(retstat);
}

int CryFuse::unlink(const path &path) {
  printf("unlink(%s)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int retstat = ::unlink(real_path.c_str());
  return errcode_map(retstat);
}

int CryFuse::rmdir(const path &path) {
  printf("rmdir(%s)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int retstat = ::rmdir(real_path.c_str());
  return errcode_map(retstat);
}

int CryFuse::symlink(const path &from, const path &to) {
  printf("symlink(%s, %s)\n", from.c_str(), to.c_str());
  auto real_from = _device->RootDir() / from;
  auto real_to = _device->RootDir() / to;
  int retstat = ::symlink(real_from.c_str(), real_to.c_str());
  return errcode_map(retstat);
}

int CryFuse::rename(const path &from, const path &to) {
  printf("rename(%s, %s)\n", from.c_str(), to.c_str());
  auto real_from = _device->RootDir() / from;
  auto real_to = _device->RootDir() / to;
  int retstat = ::rename(real_from.c_str(), real_to.c_str());
  return errcode_map(retstat);
}

int CryFuse::link(const path &from, const path &to) {
  printf("link(%s, %s)\n", from.c_str(), to.c_str());
  auto real_from = _device->RootDir() / from;
  auto real_to = _device->RootDir() / to;
  int retstat = ::link(real_from.c_str(), real_to.c_str());
  return errcode_map(retstat);
}

int CryFuse::chmod(const path &path, mode_t mode) {
  printf("chmod(%s, %d)\n", path.c_str(), mode);
  auto real_path = _device->RootDir() / path;
  int retstat = ::chmod(real_path.c_str(), mode);
  return errcode_map(retstat);
}

int CryFuse::chown(const path &path, uid_t uid, gid_t gid) {
  printf("chown(%s, %d, %d)\n", path.c_str(), uid, gid);
  auto real_path = _device->RootDir() / path;
  int retstat = ::chown(real_path.c_str(), uid, gid);
  return errcode_map(retstat);
}

int CryFuse::truncate(const path &path, off_t size) {
  printf("truncate(%s, %zu)\n", path.c_str(), size);
  auto real_path = _device->RootDir() / path;
  int retstat = ::truncate(real_path.c_str(), size);
  return errcode_map(retstat);
}

int CryFuse::ftruncate(const path &path, off_t size, fuse_file_info *fileinfo) {
  printf("ftruncate(%s, %zu, _)\n", path.c_str(), size);
  int retstat = ::ftruncate(fileinfo->fh, size);
  return errcode_map(retstat);
}

int CryFuse::utimens(const path &path, const timespec times[2]) {
  printf("utimens(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  struct timeval tv[2];
  tv[0].tv_sec = times[0].tv_sec;
  tv[0].tv_usec = times[0].tv_nsec / 1000;
  tv[1].tv_sec = times[1].tv_sec;
  tv[1].tv_usec = times[1].tv_nsec / 1000;
  int retstat = ::lutimes(real_path.c_str(), tv);
  return errcode_map(retstat);
}

int CryFuse::open(const path &path, fuse_file_info *fileinfo) {
  printf("open(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int fd = ::open(real_path.c_str(), fileinfo->flags);
  if (fd < 0) {
    return -errno;
  }
  fileinfo->fh = fd;
  return 0;
}

int CryFuse::release(const path &path, fuse_file_info *fileinfo) {
  printf("release(%s, _)\n", path.c_str());
  int retstat = ::close(fileinfo->fh);
  return errcode_map(retstat);
}

int CryFuse::read(const path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  printf("read(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  int retstat = ::pread(fileinfo->fh, buf, size, offset);
  return errcode_map(retstat);
}

int CryFuse::write(const path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  printf("write(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  int retstat = ::pwrite(fileinfo->fh, buf, size, offset);
  return errcode_map(retstat);
}

int CryFuse::statfs(const path &path, struct statvfs *fsstat) {
  printf("statfs(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int retstat = ::statvfs(real_path.c_str(), fsstat);
  return errcode_map(retstat);
}

int CryFuse::flush(const path &path, fuse_file_info *fileinfo) {
  printf("Called non-implemented flush(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::fsync(const path &path, int datasync, fuse_file_info *fileinfo) {
  printf("fsync(%s, %d, _)\n", path.c_str(), datasync);
  int retstat = 0;
  if (datasync) {
    retstat = ::fdatasync(fileinfo->fh);
  } else {
    retstat = ::fsync(fileinfo->fh);
  }
  return errcode_map(retstat);
}

int CryFuse::opendir(const path &path, fuse_file_info *fileinfo) {
  printf("opendir(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  DIR *dp = ::opendir(real_path.c_str());
  if (dp == nullptr) {
    return -errno;
  }
  fileinfo->fh = (intptr_t)dp;
  return 0;
}

int CryFuse::readdir(const path &path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  printf("readdir(%s, _, _, %zu, _)\n", path.c_str(), offset);
  auto real_path = _device->RootDir() / path;

  DIR *dp = (DIR*)(uintptr_t)fileinfo->fh;
  struct dirent *de = ::readdir(dp);
  if (de == nullptr) {
    return -errno;
  }

  do {
    if (filler(buf, de->d_name, nullptr, 0) != 0) {
      return -ENOMEM;
    }
  } while ((de = ::readdir(dp)) != nullptr);

  return 0;
}

int CryFuse::releasedir(const path &path, fuse_file_info *fileinfo) {
  printf("releasedir(%s, _)\n", path.c_str());
  int retstat = closedir((DIR*)(uintptr_t)fileinfo->fh);
  return errcode_map(retstat);
}

int CryFuse::fsyncdir(const path &path, int datasync, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented fsyncdir(%s, %d, _)\n", path.c_str(), datasync);
  return 0;
}

void CryFuse::init(fuse_conn_info *conn) {
  UNUSED(conn);
  printf("init()\n");
}

void CryFuse::destroy() {
  printf("destroy()\n");
}

int CryFuse::access(const path &path, int mask) {
  printf("access(%s, %d)\n", path.c_str(), mask);
  auto real_path = _device->RootDir() / path;
  int retstat = ::access(real_path.c_str(), mask);
  return errcode_map(retstat);
}

int CryFuse::create(const path &path, mode_t mode, fuse_file_info *fileinfo) {
  printf("create(%s, %d, _)\n", path.c_str(), mode);
  auto real_path = _device->RootDir() / path;
  int fd = ::creat(real_path.c_str(), mode);
  if (fd < 0) {
    return -errno;
  }
  fileinfo->fh = fd;
  return 0;
}

} /* namespace cryfs */
