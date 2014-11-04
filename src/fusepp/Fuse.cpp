#include "../fusepp/Fuse.h"
#include <memory>
#include <cassert>

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

void Fuse::run(int argc, char **argv) {
  fuse_main(argc, argv, operations(), (void*)this);
}
