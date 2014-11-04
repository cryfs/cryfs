#pragma once
#ifndef CRYFS_LIB_CRYFUSE_H_
#define CRYFS_LIB_CRYFUSE_H_

#include "fusepp/Fuse.h"
#include "CryDevice.h"
#include "utils/macros.h"

namespace cryfs {

class CryFuse: public fusepp::Fuse {
public:
  CryFuse(CryDevice *device);

  int getattr(const fusepp::path &path, struct stat *stbuf) override;
  int fgetattr(const fusepp::path &path, struct stat *stbuf, fuse_file_info *fileinfo) override;
  int readlink(const fusepp::path &path, char *buf, size_t size) override;
  int mknod(const fusepp::path &path, mode_t mode, dev_t rdev) override;
  int mkdir(const fusepp::path &path, mode_t mode) override;
  int unlink(const fusepp::path &path) override;
  int rmdir(const fusepp::path &path) override;
  int symlink(const fusepp::path &from, const fusepp::path &to) override;
  int rename(const fusepp::path &from, const fusepp::path &to) override;
  int link(const fusepp::path &from, const fusepp::path &to) override;
  int chmod(const fusepp::path &path, mode_t mode) override;
  int chown(const fusepp::path &path, uid_t uid, gid_t gid) override;
  int truncate(const fusepp::path &path, off_t size) override;
  int ftruncate(const fusepp::path &path, off_t size, fuse_file_info *fileinfo) override;
  int utimens(const fusepp::path &path, const timespec times[2]) override;
  int open(const fusepp::path &path, fuse_file_info *fileinfo) override;
  int release(const fusepp::path &path, fuse_file_info *fileinfo) override;
  int read(const fusepp::path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override;
  int write(const fusepp::path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) override;
  int statfs(const fusepp::path &path, struct statvfs *fsstat) override;
  int flush(const fusepp::path &path, fuse_file_info *fileinfo) override;
  int fsync(const fusepp::path &path, int flags, fuse_file_info *fileinfo) override;
  int opendir(const fusepp::path &path, fuse_file_info *fileinfo) override;
  int readdir(const fusepp::path &path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) override;
  int releasedir(const fusepp::path &path, fuse_file_info *fileinfo) override;
  int fsyncdir(const fusepp::path &path, int datasync, fuse_file_info *fileinfo) override;
  void init(fuse_conn_info *conn) override;
  void destroy() override;
  int access(const fusepp::path &path, int mask) override;
  int create(const fusepp::path &path, mode_t mode, fuse_file_info *fileinfo) override;

private:
  CryDevice *_device;

  DISALLOW_COPY_AND_ASSIGN(CryFuse);
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFUSE_H_ */
