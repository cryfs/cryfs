#pragma once
#ifndef CRYFS_LIB_FUSEPP_FUSE_H_
#define CRYFS_LIB_FUSEPP_FUSE_H_

#include "params.h"
#include <fuse.h>
#include <cstdio>
#include <string>
#include <sys/stat.h>

namespace fusepp {
//TODO If performance suffers here, we could use template<class FuseImpl>
//     and redirect the fuse calls directly to the FuseImpl class instead
//     of using virtual functions.
class Fuse {
public:
	virtual ~Fuse();

	void run(int argc, char **argv);

	virtual int getattr(const char *path, struct stat *stbuf) = 0;
	virtual int fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) = 0;
	virtual int readlink(const char *path, char *buf, size_t size) = 0;
	virtual int mknod(const char *path, mode_t mode, dev_t rdev) = 0;
	virtual int mkdir(const char *path, mode_t mode) = 0;
	virtual int unlink(const char *path) = 0;
	virtual int rmdir(const char *path) = 0;
	virtual int symlink(const char *from, const char *to) = 0;
	virtual int rename(const char *from, const char *to) = 0;
	virtual int link(const char *from, const char *to) = 0;
	virtual int chmod(const char *path, mode_t mode) = 0;
	virtual int chown(const char *path, uid_t uid, gid_t gid) = 0;
	virtual int truncate(const char *path, off_t size) = 0;
	virtual int ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) = 0;
	virtual int utimens(const char *path, const timespec times[2]) = 0;
	virtual int open(const char *path, fuse_file_info *fileinfo) = 0;
	virtual int release(const char *path, fuse_file_info *fileinfo) = 0;
	virtual int read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int statfs(const char *path, struct statvfs *fsstat) = 0;
	virtual int flush(const char *path, fuse_file_info *fileinfo) = 0;
	virtual int fsync(const char *path, int flags, fuse_file_info *fileinfo) = 0;
	virtual int opendir(const char *path, fuse_file_info *fileinfo) = 0;
	virtual int readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int releasedir(const char *path, fuse_file_info *fileinfo) = 0;
	virtual int fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) = 0;
	virtual void init(fuse_conn_info *conn) = 0;
	virtual void destroy() = 0;
	virtual int access(const char *path, int mask) = 0;
	virtual int create(const char *path, mode_t mode, fuse_file_info *fileinfo) = 0;
};
}

#endif /* CRYFS_LIB_FUSEPP_FUSE_H_ */
