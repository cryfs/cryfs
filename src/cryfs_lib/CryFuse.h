#pragma once
#ifndef CRYFS_LIB_CRYFUSE_H_
#define CRYFS_LIB_CRYFUSE_H_

#include "fusepp/Fuse.h"

namespace cryfs {

class CryFuse: public fusepp::Fuse {
public:
  int getattr(const char *path, struct stat *stbuf) override;
  int fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) override;
  int readlink(const char *path, char *buf, size_t size) override;
  int mknod(const char *path, mode_t mode, dev_t rdev) override;
  int mkdir(const char *path, mode_t mode) override;
  int unlink(const char *path) override;
  int rmdir(const char *path) override;
  int symlink(const char *from, const char *to) override;
  int rename(const char *from, const char *to) override;
  int link(const char *from, const char *to) override;
  int chmod(const char *path, mode_t mode) override;
  int chown(const char *path, uid_t uid, gid_t gid) override;
  int truncate(const char *path, off_t size) override;
  int ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) override;
  int utimens(const char *path, const timespec times[2]) override;
  int open(const char *path, fuse_file_info *fileinfo) override;
  int release(const char *path, fuse_file_info *fileinfo) override;
  int read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override;
  int write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override;
  int statfs(const char *path, struct statvfs *fsstat) override;
  int flush(const char *path, fuse_file_info *fileinfo) override;
  int fsync(const char *path, int flags, fuse_file_info *fileinfo) override;
  int opendir(const char *path, fuse_file_info *fileinfo) override;
  int readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) override;
  int releasedir(const char *path, fuse_file_info *fileinfo) override;
  int fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) override;
  void* init(fuse_conn_info *conn) override;
  void destroy(void *userdata) override;
  int access(const char *path, int mask) override;
  int create(const char *path, mode_t mode, fuse_file_info *fileinfo) override;
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFUSE_H_ */
