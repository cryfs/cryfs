#include "CryFuse.h"

#include <sys/types.h>
#include <sys/time.h>
#include <dirent.h>
#include <cassert>

#include "cryfs_lib/CryNode.h"
#include "cryfs_lib/CryErrnoException.h"

#define UNUSED(expr) (void)(expr)

using fusepp::path;

namespace cryfs {

CryFuse::CryFuse(CryDevice *device)
  :_device(device) {
}

int CryFuse::getattr(const path &path, struct stat *stbuf) {
  try {
    _device->lstat(path, stbuf);
    return 0;
  } catch(cryfs::CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::fgetattr(const path &path, struct stat *stbuf, fuse_file_info *fileinfo) {
  //printf("fgetattr(%s, _, _)\n", path.c_str());

  // On FreeBSD, trying to do anything with the mountpoint ends up
  // opening it, and then using the FD for an fgetattr.  So in the
  // special case of a path of "/", I need to do a getattr on the
  // underlying root directory instead of doing the fgetattr().
  // TODO Check if necessary
  if (path.native() == "/") {
    return getattr(path, stbuf);
  }

  try {
	_device->fstat(fileinfo->fh, stbuf);
	return 0;
  } catch(cryfs::CryErrnoException &e) {
	  return -e.getErrno();
  }
}

//TODO
int CryFuse::readlink(const path &path, char *buf, size_t size) {
  //printf("readlink(%s, _, %zu)\n", path.c_str(), size);
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
  UNUSED(mode);
  UNUSED(path);
  printf("Called non-implemented mknod(%s, %d, _)\n", path.c_str(), mode);
  return 0;
}

int CryFuse::mkdir(const path &path, mode_t mode) {
  //printf("mkdir(%s, %d)\n", path.c_str(), mode);
  try {
    _device->mkdir(path, mode);
    return 0;
  } catch(cryfs::CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::unlink(const path &path) {
  //printf("unlink(%s)\n", path.c_str());
  try {
    _device->unlink(path);
    return 0;
  } catch(cryfs::CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::rmdir(const path &path) {
  try {
    _device->rmdir(path);
    return 0;
  } catch(cryfs::CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::symlink(const path &from, const path &to) {
  printf("NOT IMPLEMENTED: symlink(%s, %s)\n", from.c_str(), to.c_str());
  //auto real_from = _device->RootDir() / from;
  //auto real_to = _device->RootDir() / to;
  //int retstat = ::symlink(real_from.c_str(), real_to.c_str());
  //return errcode_map(retstat);
  return ENOSYS;
}

int CryFuse::rename(const path &from, const path &to) {
  //printf("rename(%s, %s)\n", from.c_str(), to.c_str());
  try {
    _device->rename(from, to);
    return 0;
  } catch(cryfs::CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::link(const path &from, const path &to) {
  printf("NOT IMPLEMENTED: link(%s, %s)\n", from.c_str(), to.c_str());
  //auto real_from = _device->RootDir() / from;
  //auto real_to = _device->RootDir() / to;
  //int retstat = ::link(real_from.c_str(), real_to.c_str());
  //return errcode_map(retstat);
  return ENOSYS;
}

//TODO
int CryFuse::chmod(const path &path, mode_t mode) {
  printf("NOT IMPLEMENTED: chmod(%s, %d)\n", path.c_str(), mode);
  //auto real_path = _device->RootDir() / path;
  //int retstat = ::chmod(real_path.c_str(), mode);
  //return errcode_map(retstat);
  return ENOSYS;
}

//TODO
int CryFuse::chown(const path &path, uid_t uid, gid_t gid) {
  printf("NOT IMPLEMENTED: chown(%s, %d, %d)\n", path.c_str(), uid, gid);
  //auto real_path = _device->RootDir() / path;
  //int retstat = ::chown(real_path.c_str(), uid, gid);
  //return errcode_map(retstat);
  return ENOSYS;
}

int CryFuse::truncate(const path &path, off_t size) {
  //printf("truncate(%s, %zu)\n", path.c_str(), size);
  try {
    _device->truncate(path, size);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::ftruncate(const path &path, off_t size, fuse_file_info *fileinfo) {
  //printf("ftruncate(%s, %zu, _)\n", path.c_str(), size);
	UNUSED(path);
  try {
    _device->ftruncate(fileinfo->fh, size);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::utimens(const path &path, const timespec times[2]) {
  //printf("utimens(%s, _)\n", path.c_str());
  try {
    _device->utimens(path, times);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::open(const path &path, fuse_file_info *fileinfo) {
  //printf("open(%s, _)\n", path.c_str());
  try {
	  fileinfo->fh = _device->openFile(path, fileinfo->flags);
	  return 0;
  } catch (CryErrnoException &e) {
	  return -e.getErrno();
  }
}

int CryFuse::release(const path &path, fuse_file_info *fileinfo) {
  //printf("release(%s, _)\n", path.c_str());
  UNUSED(path);
  try {
	  _device->closeFile(fileinfo->fh);
	  return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::read(const path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  //printf("read(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  UNUSED(path);
  try {
    //printf("Reading from file %d\n", fileinfo->fh);
    //fflush(stdout);
    return _device->read(fileinfo->fh, buf, size, offset);
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::write(const path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  //printf("write(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  UNUSED(path);
  try {
    _device->write(fileinfo->fh, buf, size, offset);
    return size;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::statfs(const path &path, struct statvfs *fsstat) {
  //printf("statfs(%s, _)\n", path.c_str());
  try {
    _device->statfs(path, fsstat);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::flush(const path &path, fuse_file_info *fileinfo) {
  //printf("Called non-implemented flush(%s, _)\n", path.c_str());
  UNUSED(path);
  UNUSED(fileinfo);
  return 0;
}

int CryFuse::fsync(const path &path, int datasync, fuse_file_info *fileinfo) {
  //printf("fsync(%s, %d, _)\n", path.c_str(), datasync);
  UNUSED(path);
  try {
    if (datasync) {
      _device->fdatasync(fileinfo->fh);
    } else {
      _device->fsync(fileinfo->fh);
    }
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::opendir(const path &path, fuse_file_info *fileinfo) {
  //printf("opendir(%s, _)\n", path.c_str());
  try {
    fileinfo->fh = _device->openDir(path);
    return 0;
  } catch(CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::readdir(const path &path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(path);
  //printf("readdir(%s, _, _, %zu, _)\n", path.c_str(), offset);
  UNUSED(offset);
  try {
    auto entries = _device->readDir(fileinfo->fh);
    for (const auto &entry : *entries) {
      //TODO Also give file attributes (third param of filler)
      if (filler(buf, entry.c_str(), nullptr, 0) != 0) {
        return -ENOMEM;
      }
    }
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::releasedir(const path &path, fuse_file_info *fileinfo) {
  //printf("releasedir(%s, _)\n", path.c_str());
  UNUSED(path);
  try {
    _device->closeDir(fileinfo->fh);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int CryFuse::fsyncdir(const path &path, int datasync, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  UNUSED(datasync);
  UNUSED(path);
  //printf("Called non-implemented fsyncdir(%s, %d, _)\n", path.c_str(), datasync);
  return 0;
}

void CryFuse::init(fuse_conn_info *conn) {
  UNUSED(conn);
  //printf("init()\n");
}

void CryFuse::destroy() {
  //printf("destroy()\n");
}

int CryFuse::access(const path &path, int mask) {
  //printf("access(%s, %d)\n", path.c_str(), mask);
  try {
    _device->access(path, mask);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

int CryFuse::create(const path &path, mode_t mode, fuse_file_info *fileinfo) {
  //printf("create(%s, %d, _)\n", path.c_str(), mode);
  try {
    fileinfo->fh = _device->createAndOpenFile(path, mode);
    return 0;
  } catch (CryErrnoException &e) {
    return -e.getErrno();
  }
}

} /* namespace cryfs */
