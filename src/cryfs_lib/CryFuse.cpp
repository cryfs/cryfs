#include "CryFuse.h"

#include <sys/types.h>
#include <dirent.h>
#include <cassert>

#define UNUSED(expr) (void)(expr)

using fusepp::path;

namespace cryfs {

CryFuse::CryFuse(CryDevice *device)
  :_device(device) {
}

int CryFuse::getattr(const path &path, struct stat *stbuf) {
  UNUSED(stbuf);
  printf("getattr(%s, _)\n", path.c_str());
  auto real_path = _device->RootDir() / path;
  int retstat = lstat(real_path.c_str(), stbuf);
  if (retstat != 0) {
    return -errno;
  }
  return 0;
}

int CryFuse::fgetattr(const path &path, struct stat *stbuf, fuse_file_info *fileinfo) {
  UNUSED(stbuf);
  UNUSED(fileinfo);
  printf("Called non-implemented fgetattr(%s, _, _)\n", path.c_str());
  return 0;
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
  printf("Called non-implemented mkdir(%s, %d)\n", path.c_str(), mode);
  return 0;
}

int CryFuse::unlink(const path &path) {
  printf("Called non-implemented unlink(%s)\n", path.c_str());
  return 0;
}

int CryFuse::rmdir(const path &path) {
  printf("Called non-implemented rmdir(%s)\n", path.c_str());
  return 0;
}

int CryFuse::symlink(const path &from, const path &to) {
  printf("Called non-implemented symlink(%s, %s)\n", from.c_str(), to.c_str());
  return 0;
}

int CryFuse::rename(const path &from, const path &to) {
  printf("Called non-implemented rename(%s, %s)\n", from.c_str(), to.c_str());
  return 0;
}

int CryFuse::link(const path &from, const path &to) {
  printf("Called non-implemented link(%s, %s)\n", from.c_str(), to.c_str());
  return 0;
}

int CryFuse::chmod(const path &path, mode_t mode) {
  printf("Called non-implemented chmod(%s, %d)\n", path.c_str(), mode);
  return 0;
}

int CryFuse::chown(const path &path, uid_t uid, gid_t gid) {
  printf("Called non-implemented chown(%s, %d, %d)\n", path.c_str(), uid, gid);
  return 0;
}

int CryFuse::truncate(const path &path, off_t size) {
  printf("Called non-implemented truncate(%s, %zu)\n", path.c_str(), size);
  return 0;
}

int CryFuse::ftruncate(const path &path, off_t size, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented ftruncate(%s, %zu, _)\n", path.c_str(), size);
  return 0;
}

int CryFuse::utimens(const path &path, const timespec times[2]) {
  UNUSED(times);
  printf("Called non-implemented utimens(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::open(const path &path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented open(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::release(const path &path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented release(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::read(const path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(buf);
  UNUSED(fileinfo);
  printf("Called non-implemented read(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  return 0;
}

int CryFuse::write(const path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(buf);
  UNUSED(fileinfo);
  printf("Called non-implemented write(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  return 0;
}

int CryFuse::statfs(const path &path, struct statvfs *fsstat) {
  UNUSED(fsstat);
  printf("Called non-implemented statfs(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::flush(const path &path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented flush(%s, _)\n", path.c_str());
  return 0;
}

int CryFuse::fsync(const path &path, int flags, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented fsync(%s, %d, _)\n", path.c_str(), flags);
  return 0;
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
  if (retstat != 0) {
    return -errno;
  }
  return 0;
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
  printf("Called non-implemented access(%s, %d)\n", path.c_str(), mask);
  return 0;
}

int CryFuse::create(const path &path, mode_t mode, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented create(%s, %d, _)\n", path.c_str(), mode);
  return 0;
}

} /* namespace cryfs */
