#pragma once
#ifndef CRYFS_LIB_CRYFUSE_H_
#define CRYFS_LIB_CRYFUSE_H_

#include "fusepp/Fuse.h"

namespace cryfs {

class CryFuse: public fusepp::Fuse {
public:
  int getattr(const char *path, struct stat *stbuf) override {
    printf("Called non-implemented getattr(%s, _)\n", path);
    return 0;
  }

  int fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) override {
    printf("Called non-implemented fgetattr(%s, _, _)\n", path);
    return 0;
  }

  int readlink(const char *path, char *buf, size_t size) override {
    printf("Called non-implemented readlink(%s, _, %d)\n", path, size);
    return 0;
  }

  int mknod(const char *path, mode_t mode, dev_t rdev) override {
    printf("Called non-implemented mknod(%s, %d, _)\n", path, mode);
    return 0;
  }

  int mkdir(const char *path, mode_t mode) override {
    printf("Called non-implemented mkdir(%s, %d)\n", path, mode);
    return 0;
  }

  int unlink(const char *path) override {
    printf("Called non-implemented unlink(%s)\n", path);
    return 0;
  }

  int rmdir(const char *path) override {
    printf("Called non-implemented rmdir(%s)\n", path);
    return 0;
  }

  int symlink(const char *from, const char *to) override {
    printf("Called non-implemented symlink(%s, %s)\n", from, to);
    return 0;
  }

  int rename(const char *from, const char *to) override {
    printf("Called non-implemented rename(%s, %s)\n", from, to);
    return 0;
  }

  int link(const char *from, const char *to) override {
    printf("Called non-implemented link(%s, %s)\n", from, to);
    return 0;
  }

  int chmod(const char *path, mode_t mode) override {
    printf("Called non-implemented chmod(%s, %d)\n", path, mode);
    return 0;
  }

  int chown(const char *path, uid_t uid, gid_t gid) override {
    printf("Called non-implemented chown(%s, %d, %d)\n", path, uid, gid);
    return 0;
  }

  int truncate(const char *path, off_t size) override {
    printf("Called non-implemented truncate(%s, %d)\n", path, size);
    return 0;
  }

  int ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) override {
    printf("Called non-implemented ftruncate(%s, %d)\n", path, size);
    return 0;
  }

  int utimens(const char *path, const timespec times[2]) override {
    printf("Called non-implemented utimens(%s, _)\n", path);
    return 0;
  }

  int open(const char *path, fuse_file_info *fileinfo) override {
    printf("Called non-implemented open(%s)\n", path);
    return 0;
  }

  int release(const char *path, fuse_file_info *fileinfo) override {
    printf("Called non-implemented release(%s)\n", path);
    return 0;
  }

  int read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override {
    printf("Called non-implemented read(%s, _, %d, %d, _)\n", path, size, offset);
    return 0;
  }

  int write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override {
    printf("Called non-implemented write(%s, _, %d, %d, _)\n", path, size, offset);
    return 0;
  }

  int statfs(const char *path, struct statvfs *fsstat) override {
    printf("Called non-implemented statfs(%s)\n", path);
    return 0;
  }

  int flush(const char *path, fuse_file_info *fileinfo) override {
    printf("Called non-implemented flush(%s)\n", path);
    return 0;
  }

  int fsync(const char *path, int flags, fuse_file_info *fileinfo) override {
    printf("Called non-implemented fsync(%s, %d, _)\n", path, flags);
    return 0;
  }

  int opendir(const char *path, fuse_file_info *fileinfo) override {
    printf("Called non-implemented opendir(%s, _)\n", path);
    return 0;
  }

  int readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) override {
    printf("Called non-implemented readdir(%s, _, _, %d, _)\n", path, offset);
    return 0;
  }

  int releasedir(const char *path, fuse_file_info *fileinfo) override {
    printf("Called non-implemented releasedir(%s, _)\n", path);
    return 0;
  }

  int fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) override {
    printf("Called non-implemented fsyncdir(%s, %d, _)\n", path, datasync);
    return 0;
  }

  void* init(fuse_conn_info *conn) override {
    printf("Called non-implemented init()\n");
    return this;
  }

  void destroy(void *userdata) override {
    printf("Called non-implemented destroy()\n");
  }

  int access(const char *path, int mask) override {
    printf("Called non-implemented access(%s, %d)\n", path, mask);
    return 0;
  }

  int create(const char *path, mode_t mode, fuse_file_info *fileinfo) override {
    printf("Called non-implemented create(%s, %d, _)\n", path, mode);
    return 0;
  }
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFUSE_H_ */
