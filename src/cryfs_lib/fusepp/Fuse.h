#pragma once
#ifndef CRYFS_LIB_FUSEPP_FUSE_H_
#define CRYFS_LIB_FUSEPP_FUSE_H_

#include "params.h"
#include <fuse.h>
#include <cstdio>
#include <string>
#include <sys/stat.h>
#include <boost/filesystem.hpp>

namespace fusepp {

typedef boost::filesystem::path path;

//TODO If performance suffers here, we could use template<class FuseImpl>
//     and redirect the fuse calls directly to the FuseImpl class instead
//     of using virtual functions.
class Fuse {
public:
	virtual ~Fuse();

	void run(int argc, char **argv);

	virtual int getattr(const path &path, struct stat *stbuf) = 0;
	virtual int fgetattr(const path &path, struct stat *stbuf, fuse_file_info *fileinfo) = 0;
	virtual int readlink(const path &path, char *buf, size_t size) = 0;
	virtual int mknod(const path &path, mode_t mode, dev_t rdev) = 0;
	virtual int mkdir(const path &path, mode_t mode) = 0;
	virtual int unlink(const path &path) = 0;
	virtual int rmdir(const path &path) = 0;
	virtual int symlink(const path &from, const path &to) = 0;
	virtual int rename(const path &from, const path &to) = 0;
	virtual int link(const path &from, const path &to) = 0;
	virtual int chmod(const path &path, mode_t mode) = 0;
	virtual int chown(const path &path, uid_t uid, gid_t gid) = 0;
	virtual int truncate(const path &path, off_t size) = 0;
	virtual int ftruncate(const path &path, off_t size, fuse_file_info *fileinfo) = 0;
	virtual int utimens(const path &path, const timespec times[2]) = 0;
	virtual int open(const path &path, fuse_file_info *fileinfo) = 0;
	virtual int release(const path &path, fuse_file_info *fileinfo) = 0;
	virtual int read(const path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int write(const path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int statfs(const path &path, struct statvfs *fsstat) = 0;
	virtual int flush(const path &path, fuse_file_info *fileinfo) = 0;
	virtual int fsync(const path &path, int flags, fuse_file_info *fileinfo) = 0;
	virtual int opendir(const path &path, fuse_file_info *fileinfo) = 0;
	virtual int readdir(const path &path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) = 0;
	virtual int releasedir(const path &path, fuse_file_info *fileinfo) = 0;
	virtual int fsyncdir(const path &path, int datasync, fuse_file_info *fileinfo) = 0;
	virtual void init(fuse_conn_info *conn) = 0;
	virtual void destroy() = 0;
	virtual int access(const path &path, int mask) = 0;
	virtual int create(const path &path, mode_t mode, fuse_file_info *fileinfo) = 0;
};
}

#endif /* CRYFS_LIB_FUSEPP_FUSE_H_ */
