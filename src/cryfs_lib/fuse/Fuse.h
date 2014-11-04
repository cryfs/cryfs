#pragma once
#ifndef CRYFS_LIB_FUSE_FUSE_H_
#define CRYFS_LIB_FUSE_FUSE_H_

#include "params.h"
#include <fuse.h>
#include <cstdio>
#include <string>
#include <sys/stat.h>

class Fuse {
public:
	virtual ~Fuse();

	void run(int argc, char **argv);

	int getattr(const char *path, struct stat *stbuf) {
	  printf("Called getattr(%s, _)\n", path);
	  return 0;
	}

	int fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) {
    printf("Called fgetattr(%s, _, _)\n", path);
    return 0;
	}

	int readlink(const char *path, char *buf, size_t size) {
    printf("Called readlink(%s, _, %d)\n", path, size);
    return 0;
	}

	int mknod(const char *path, mode_t mode, dev_t rdev) {
    printf("Called mknod(%s, %d, _)\n", path, mode);
    return 0;
	}

	int mkdir(const char *path, mode_t mode) {
    printf("Called mkdir(%s, %d)\n", path, mode);
    return 0;
	}

	int unlink(const char *path) {
    printf("Called unlink(%s)\n", path);
    return 0;
	}

	int rmdir(const char *path) {
    printf("Called rmdir(%s)\n", path);
    return 0;
	}

	int symlink(const char *from, const char *to) {
    printf("Called symlink(%s, %s)\n", from, to);
    return 0;
	}

	int rename(const char *from, const char *to) {
    printf("Called rename(%s, %s)\n", from, to);
    return 0;
	}

	int link(const char *from, const char *to) {
    printf("Called link(%s, %s)\n", from, to);
    return 0;
	}

	int chmod(const char *path, mode_t mode) {
    printf("Called chmod(%s, %d)\n", path, mode);
    return 0;
	}

	int chown(const char *path, uid_t uid, gid_t gid) {
    printf("Called chown(%s, %d, %d)\n", path, uid, gid);
    return 0;
	}

	int truncate(const char *path, off_t size) {
    printf("Called truncate(%s, %d)\n", path, size);
    return 0;
	}

	int ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) {
    printf("Called ftruncate(%s, %d)\n", path, size);
    return 0;
	}

	int utimens(const char *path, const timespec times[2]) {
    printf("Called utimens(%s, _)\n", path);
    return 0;
  }

	int open(const char *path, fuse_file_info *fileinfo) {
    printf("Called open(%s)\n", path);
    return 0;
  }

	int release(const char *path, fuse_file_info *fileinfo) {
    printf("Called release(%s)\n", path);
    return 0;
  }

	int read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
    printf("Called read(%s, _, %d, %d, _)\n", path, size, offset);
    return 0;
  }

	int write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
    printf("Called write(%s, _, %d, %d, _)\n", path, size, offset);
    return 0;
  }

	int statfs(const char *path, struct statvfs *fsstat) {
    printf("Called statfs(%s)\n", path);
    return 0;
  }

	int flush(const char *path, fuse_file_info *fileinfo) {
    printf("Called flush(%s)\n", path);
    return 0;
  }

	int fsync(const char *path, int flags, fuse_file_info *fileinfo) {
    printf("Called fsync(%s, %d, _)\n", path, flags);
    return 0;
  }

	int opendir(const char *path, fuse_file_info *fileinfo) {
    printf("Called opendir(%s, _)\n", path);
    return 0;
  }

	int readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
    printf("Called readdir(%s, _, _, %d, _)\n", path, offset);
    return 0;
  }

	int releasedir(const char *path, fuse_file_info *fileinfo) {
    printf("Called releasedir(%s, _)\n", path);
    return 0;
  }

	int fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) {
    printf("Called fsyncdir(%s, %d, _)\n", path, datasync);
    return 0;
  }

	void* init(fuse_conn_info *conn) {
    printf("Called init()\n");
    return this;
  }

	void destroy(void *userdata) {
    printf("Called destroy()\n");
  }

	int access(const char *path, int mask) {
    printf("Called access(%s, %d)\n", path, mask);
    return 0;
  }

	int create(const char *path, mode_t mode, fuse_file_info *fileinfo) {
    printf("Called create(%s, %d, _)\n", path, mode);
    return 0;
  }

};

#endif /* CRYFS_LIB_FUSE_FUSE_H_ */
