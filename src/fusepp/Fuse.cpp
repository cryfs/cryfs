#include "Fuse.h"
#include <memory>
#include <cassert>

#include "FuseDevice.h"
#include "FuseErrnoException.h"

using std::unique_ptr;
using std::make_unique;
using std::string;

namespace bf = boost::filesystem;

using namespace fusepp;

#define FUSE_OBJ ((Fuse *) fuse_get_context()->private_data)

namespace {
int fusepp_getattr(const char *path, struct stat *stbuf) {
  return FUSE_OBJ->getattr(bf::path(path), stbuf);
}

int fusepp_fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fgetattr(bf::path(path), stbuf, fileinfo);
}

int fusepp_readlink(const char *path, char *buf, size_t size) {
  return FUSE_OBJ->readlink(bf::path(path), buf, size);
}

int fusepp_mknod(const char *path, mode_t mode, dev_t rdev) {
  return FUSE_OBJ->mknod(bf::path(path), mode, rdev);
}

int fusepp_mkdir(const char *path, mode_t mode) {
  return FUSE_OBJ->mkdir(bf::path(path), mode);
}

int fusepp_unlink(const char *path) {
  return FUSE_OBJ->unlink(bf::path(path));
}

int fusepp_rmdir(const char *path) {
  return FUSE_OBJ->rmdir(bf::path(path));
}

int fusepp_symlink(const char *from, const char *to) {
  return FUSE_OBJ->symlink(bf::path(from), bf::path(to));
}

int fusepp_rename(const char *from, const char *to) {
  return FUSE_OBJ->rename(bf::path(from), bf::path(to));
}

int fusepp_link(const char *from, const char *to) {
  return FUSE_OBJ->link(bf::path(from), bf::path(to));
}

int fusepp_chmod(const char *path, mode_t mode) {
  return FUSE_OBJ->chmod(bf::path(path), mode);
}

int fusepp_chown(const char *path, uid_t uid, gid_t gid) {
  return FUSE_OBJ->chown(bf::path(path), uid, gid);
}

int fusepp_truncate(const char *path, off_t size) {
  return FUSE_OBJ->truncate(bf::path(path), size);
}

int fusepp_ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) {
  return FUSE_OBJ->ftruncate(bf::path(path), size, fileinfo);
}

int fusepp_utimens(const char *path, const timespec times[2]) {
  return FUSE_OBJ->utimens(bf::path(path), times);
}

int fusepp_open(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->open(bf::path(path), fileinfo);
}

int fusepp_release(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->release(bf::path(path), fileinfo);
}

int fusepp_read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->read(bf::path(path), buf, size, offset, fileinfo);
}

int fusepp_write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->write(bf::path(path), buf, size, offset, fileinfo);
}

int fusepp_statfs(const char *path, struct statvfs *fsstat) {
  return FUSE_OBJ->statfs(bf::path(path), fsstat);
}

int fusepp_flush(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->flush(bf::path(path), fileinfo);
}

int fusepp_fsync(const char *path, int datasync, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsync(bf::path(path), datasync, fileinfo);
}

//int fusepp_setxattr(const char*, const char*, const char*, size_t, int)
//int fusepp_getxattr(const char*, const char*, char*, size_t)
//int fusepp_listxattr(const char*, char*, size_t)
//int fusepp_removexattr(const char*, const char*)

int fusepp_opendir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->opendir(bf::path(path), fileinfo);
}

int fusepp_readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->readdir(bf::path(path), buf, filler, offset, fileinfo);
}

int fusepp_releasedir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->releasedir(bf::path(path), fileinfo);
}

int fusepp_fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsyncdir(bf::path(path), datasync, fileinfo);
}

void* fusepp_init(fuse_conn_info *conn) {
  auto f = FUSE_OBJ;
  f->init(conn);
  return f;
}

void fusepp_destroy(void *userdata) {
  auto f = FUSE_OBJ;
  assert(userdata == f);
  UNUSED(userdata); //In the Release build, the assert doesn't run
  f->destroy();
}

int fusepp_access(const char *path, int mask) {
  return FUSE_OBJ->access(bf::path(path), mask);
}

int fusepp_create(const char *path, mode_t mode, fuse_file_info *fileinfo) {
  return FUSE_OBJ->create(bf::path(path), mode, fileinfo);
}

/*int fusepp_lock(const char*, fuse_file_info*, int cmd, flock*)
int fusepp_bmap(const char*, size_t blocksize, uint64_t *idx)
int fusepp_ioctl(const char*, int cmd, void *arg, fuse_file_info*, unsigned int flags, void *data)
int fusepp_poll(const char*, fuse_file_info*, fuse_pollhandle *ph, unsigned *reventsp)
int fusepp_write_buf(const char*, fuse_bufvec *buf, off_t off, fuse_file_info*)
int fusepp_read_buf(const chas*, struct fuse_bufvec **bufp, size_t size, off_T off, fuse_file_info*)
int fusepp_flock(const char*, fuse_file_info*, int op)
int fusepp_fallocate(const char*, int, off_t, off_t, fuse_file_info*)*/

fuse_operations *operations() {
  static unique_ptr<fuse_operations> singleton(nullptr);

  if (!singleton) {
    singleton = make_unique<fuse_operations>();
    singleton->getattr = &fusepp_getattr;
    singleton->fgetattr = &fusepp_fgetattr;
    singleton->readlink = &fusepp_readlink;
    singleton->mknod = &fusepp_mknod;
    singleton->mkdir = &fusepp_mkdir;
    singleton->unlink = &fusepp_unlink;
    singleton->rmdir = &fusepp_rmdir;
    singleton->symlink = &fusepp_symlink;
    singleton->rename = &fusepp_rename;
    singleton->link = &fusepp_link;
    singleton->chmod = &fusepp_chmod;
    singleton->chown = &fusepp_chown;
    singleton->truncate = &fusepp_truncate;
    singleton->utimens = &fusepp_utimens;
    singleton->open = &fusepp_open;
    singleton->read = &fusepp_read;
    singleton->write = &fusepp_write;
    singleton->statfs = &fusepp_statfs;
    singleton->flush = &fusepp_flush;
    singleton->release = &fusepp_release;
    singleton->fsync = &fusepp_fsync;
  /*#ifdef HAVE_SYS_XATTR_H
    singleton->setxattr = &fusepp_setxattr;
    singleton->getxattr = &fusepp_getxattr;
    singleton->listxattr = &fusepp_listxattr;
    singleton->removexattr = &fusepp_removexattr;
  #endif*/
    singleton->opendir = &fusepp_opendir;
    singleton->readdir = &fusepp_readdir;
    singleton->releasedir = &fusepp_releasedir;
    singleton->fsyncdir = &fusepp_fsyncdir;
    singleton->init = &fusepp_init;
    singleton->destroy = &fusepp_destroy;
    singleton->access = &fusepp_access;
    singleton->create = &fusepp_create;
    singleton->ftruncate = &fusepp_ftruncate;
  }

  return singleton.get();
}
}

Fuse::~Fuse() {
}

Fuse::Fuse(FuseDevice *device)
  :_device(device) {
}

void Fuse::run(int argc, char **argv) {
  fuse_main(argc, argv, operations(), (void*)this);
}

int Fuse::getattr(const bf::path &path, struct stat *stbuf) {
  //printf("getattr(%s, _, _)\n", path.c_str());
  try {
    _device->lstat(path, stbuf);
    return 0;
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::fgetattr(const bf::path &path, struct stat *stbuf, fuse_file_info *fileinfo) {
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
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::readlink(const bf::path &path, char *buf, size_t size) {
  UNUSED(path);
  UNUSED(buf);
  UNUSED(size);
  printf("Called non-implemented readlink(%s, _, %zu)\n", path.c_str(), size);
  return ENOSYS;
}

int Fuse::mknod(const bf::path &path, mode_t mode, dev_t rdev) {
  UNUSED(rdev);
  UNUSED(mode);
  UNUSED(path);
  printf("Called non-implemented mknod(%s, %d, _)\n", path.c_str(), mode);
  return ENOSYS;
}

int Fuse::mkdir(const bf::path &path, mode_t mode) {
  //printf("mkdir(%s, %d)\n", path.c_str(), mode);
  try {
    _device->mkdir(path, mode);
    return 0;
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::unlink(const bf::path &path) {
  //printf("unlink(%s)\n", path.c_str());
  try {
    _device->unlink(path);
    return 0;
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::rmdir(const bf::path &path) {
  try {
    _device->rmdir(path);
    return 0;
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::symlink(const bf::path &from, const bf::path &to) {
  printf("NOT IMPLEMENTED: symlink(%s, %s)\n", from.c_str(), to.c_str());
  //auto real_from = _device->RootDir() / from;
  //auto real_to = _device->RootDir() / to;
  //int retstat = ::symlink(real_from.c_str(), real_to.c_str());
  //return errcode_map(retstat);
  return ENOSYS;
}

int Fuse::rename(const bf::path &from, const bf::path &to) {
  //printf("rename(%s, %s)\n", from.c_str(), to.c_str());
  try {
    _device->rename(from, to);
    return 0;
  } catch(fusepp::FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::link(const bf::path &from, const bf::path &to) {
  printf("NOT IMPLEMENTED: link(%s, %s)\n", from.c_str(), to.c_str());
  //auto real_from = _device->RootDir() / from;
  //auto real_to = _device->RootDir() / to;
  //int retstat = ::link(real_from.c_str(), real_to.c_str());
  //return errcode_map(retstat);
  return ENOSYS;
}

//TODO
int Fuse::chmod(const bf::path &path, mode_t mode) {
  printf("NOT IMPLEMENTED: chmod(%s, %d)\n", path.c_str(), mode);
  //auto real_path = _device->RootDir() / path;
  //int retstat = ::chmod(real_path.c_str(), mode);
  //return errcode_map(retstat);
  return ENOSYS;
}

//TODO
int Fuse::chown(const bf::path &path, uid_t uid, gid_t gid) {
  printf("NOT IMPLEMENTED: chown(%s, %d, %d)\n", path.c_str(), uid, gid);
  //auto real_path = _device->RootDir() / path;
  //int retstat = ::chown(real_path.c_str(), uid, gid);
  //return errcode_map(retstat);
  return ENOSYS;
}

int Fuse::truncate(const bf::path &path, off_t size) {
  //printf("truncate(%s, %zu)\n", path.c_str(), size);
  try {
    _device->truncate(path, size);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::ftruncate(const bf::path &path, off_t size, fuse_file_info *fileinfo) {
  //printf("ftruncate(%s, %zu, _)\n", path.c_str(), size);
  UNUSED(path);
  try {
    _device->ftruncate(fileinfo->fh, size);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::utimens(const bf::path &path, const timespec times[2]) {
  //printf("utimens(%s, _)\n", path.c_str());
  try {
    _device->utimens(path, times);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::open(const bf::path &path, fuse_file_info *fileinfo) {
  //printf("open(%s, _)\n", path.c_str());
  try {
    fileinfo->fh = _device->openFile(path, fileinfo->flags);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::release(const bf::path &path, fuse_file_info *fileinfo) {
  //printf("release(%s, _)\n", path.c_str());
  UNUSED(path);
  try {
    _device->closeFile(fileinfo->fh);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::read(const bf::path &path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  //printf("read(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  UNUSED(path);
  try {
    //printf("Reading from file %d\n", fileinfo->fh);
    //fflush(stdout);
    return _device->read(fileinfo->fh, buf, size, offset);
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::write(const bf::path &path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  //printf("write(%s, _, %zu, %zu, _)\n", path.c_str(), size, offset);
  UNUSED(path);
  try {
    _device->write(fileinfo->fh, buf, size, offset);
    return size;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::statfs(const bf::path &path, struct statvfs *fsstat) {
  //printf("statfs(%s, _)\n", path.c_str());
  try {
    _device->statfs(path, fsstat);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

//TODO
int Fuse::flush(const bf::path &path, fuse_file_info *fileinfo) {
  //printf("Called non-implemented flush(%s, _)\n", path.c_str());
  UNUSED(path);
  UNUSED(fileinfo);
  return 0;
}

int Fuse::fsync(const bf::path &path, int datasync, fuse_file_info *fileinfo) {
  //printf("fsync(%s, %d, _)\n", path.c_str(), datasync);
  UNUSED(path);
  try {
    if (datasync) {
      _device->fdatasync(fileinfo->fh);
    } else {
      _device->fsync(fileinfo->fh);
    }
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::opendir(const bf::path &path, fuse_file_info *fileinfo) {
  UNUSED(path);
  UNUSED(fileinfo);
  //printf("opendir(%s, _)\n", path.c_str());
  //We don't need opendir, because readdir works directly on the path
  return 0;
}

int Fuse::readdir(const bf::path &path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  //printf("readdir(%s, _, _, %zu, _)\n", path.c_str(), offset);
  UNUSED(offset);
  try {
    auto entries = _device->readDir(path);
    for (const auto &entry : *entries) {
      //We could pass file metadata to filler() in its third parameter,
      //but it doesn't help performance since fuse seems to ignore it.
      //It does getattr() calls on all entries nevertheless.
      if (filler(buf, entry.c_str(), nullptr, 0) != 0) {
        return -ENOMEM;
      }
    }
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::releasedir(const bf::path &path, fuse_file_info *fileinfo) {
  UNUSED(path);
  UNUSED(fileinfo);
  //printf("releasedir(%s, _)\n", path.c_str());
  //We don't need releasedir, because readdir works directly on the path
  return 0;
}

//TODO
int Fuse::fsyncdir(const bf::path &path, int datasync, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  UNUSED(datasync);
  UNUSED(path);
  //printf("Called non-implemented fsyncdir(%s, %d, _)\n", path.c_str(), datasync);
  return 0;
}

void Fuse::init(fuse_conn_info *conn) {
  UNUSED(conn);
  //printf("init()\n");
}

void Fuse::destroy() {
  //printf("destroy()\n");
}

int Fuse::access(const bf::path &path, int mask) {
  //printf("access(%s, %d)\n", path.c_str(), mask);
  try {
    _device->access(path, mask);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}

int Fuse::create(const bf::path &path, mode_t mode, fuse_file_info *fileinfo) {
  //printf("create(%s, %d, _)\n", path.c_str(), mode);
  try {
    fileinfo->fh = _device->createAndOpenFile(path, mode);
    return 0;
  } catch (FuseErrnoException &e) {
    return -e.getErrno();
  }
}
