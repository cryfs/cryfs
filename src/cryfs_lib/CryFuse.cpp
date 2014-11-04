#include "CryFuse.h"

#define UNUSED(expr) (void)(expr)

namespace cryfs {

int CryFuse::getattr(const char *path, struct stat *stbuf) {
  UNUSED(stbuf);
  printf("Called non-implemented getattr(%s, _)\n", path);
  return 0;
}

int CryFuse::fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) {
  UNUSED(stbuf);
  UNUSED(fileinfo);
  printf("Called non-implemented fgetattr(%s, _, _)\n", path);
  return 0;
}

int CryFuse::readlink(const char *path, char *buf, size_t size) {
  UNUSED(buf);
  printf("Called non-implemented readlink(%s, _, %zu)\n", path, size);
  return 0;
}

int CryFuse::mknod(const char *path, mode_t mode, dev_t rdev) {
  UNUSED(rdev);
  printf("Called non-implemented mknod(%s, %d, _)\n", path, mode);
  return 0;
}

int CryFuse::mkdir(const char *path, mode_t mode) {
  printf("Called non-implemented mkdir(%s, %d)\n", path, mode);
  return 0;
}

int CryFuse::unlink(const char *path) {
  printf("Called non-implemented unlink(%s)\n", path);
  return 0;
}

int CryFuse::rmdir(const char *path) {
  printf("Called non-implemented rmdir(%s)\n", path);
  return 0;
}

int CryFuse::symlink(const char *from, const char *to) {
  printf("Called non-implemented symlink(%s, %s)\n", from, to);
  return 0;
}

int CryFuse::rename(const char *from, const char *to) {
  printf("Called non-implemented rename(%s, %s)\n", from, to);
  return 0;
}

int CryFuse::link(const char *from, const char *to) {
  printf("Called non-implemented link(%s, %s)\n", from, to);
  return 0;
}

int CryFuse::chmod(const char *path, mode_t mode) {
  printf("Called non-implemented chmod(%s, %d)\n", path, mode);
  return 0;
}

int CryFuse::chown(const char *path, uid_t uid, gid_t gid) {
  printf("Called non-implemented chown(%s, %d, %d)\n", path, uid, gid);
  return 0;
}

int CryFuse::truncate(const char *path, off_t size) {
  printf("Called non-implemented truncate(%s, %zu)\n", path, size);
  return 0;
}

int CryFuse::ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented ftruncate(%s, %zu, _)\n", path, size);
  return 0;
}

int CryFuse::utimens(const char *path, const timespec times[2]) {
  UNUSED(times);
  printf("Called non-implemented utimens(%s, _)\n", path);
  return 0;
}

int CryFuse::open(const char *path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented open(%s, _)\n", path);
  return 0;
}

int CryFuse::release(const char *path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented release(%s, _)\n", path);
  return 0;
}

int CryFuse::read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(buf);
  UNUSED(fileinfo);
  printf("Called non-implemented read(%s, _, %zu, %zu, _)\n", path, size, offset);
  return 0;
}

int CryFuse::write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(buf);
  UNUSED(fileinfo);
  printf("Called non-implemented write(%s, _, %zu, %zu, _)\n", path, size, offset);
  return 0;
}

int CryFuse::statfs(const char *path, struct statvfs *fsstat) {
  UNUSED(fsstat);
  printf("Called non-implemented statfs(%s, _)\n", path);
  return 0;
}

int CryFuse::flush(const char *path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented flush(%s, _)\n", path);
  return 0;
}

int CryFuse::fsync(const char *path, int flags, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented fsync(%s, %d, _)\n", path, flags);
  return 0;
}

int CryFuse::opendir(const char *path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented opendir(%s, _)\n", path);
  return 0;
}

int CryFuse::readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(buf);
  UNUSED(filler);
  UNUSED(fileinfo);
  printf("Called non-implemented readdir(%s, _, _, %zu, _)\n", path, offset);
  return 0;
}

int CryFuse::releasedir(const char *path, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented releasedir(%s, _)\n", path);
  return 0;
}

int CryFuse::fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented fsyncdir(%s, %d, _)\n", path, datasync);
  return 0;
}

void* CryFuse::init(fuse_conn_info *conn) {
  UNUSED(conn);
  printf("Called non-implemented init()\n");
  return this;
}

void CryFuse::destroy(void *userdata) {
  UNUSED(userdata);
  printf("Called non-implemented destroy()\n");
}

int CryFuse::access(const char *path, int mask) {
  printf("Called non-implemented access(%s, %d)\n", path, mask);
  return 0;
}

int CryFuse::create(const char *path, mode_t mode, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  printf("Called non-implemented create(%s, %d, _)\n", path, mode);
  return 0;
}

} /* namespace cryfs */
