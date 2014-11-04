#include "Fuse.h"

#include <memory>

using std::unique_ptr;
using std::make_unique;
using std::string;

#define FUSE_OBJ ((Fuse *) fuse_get_context()->private_data)

namespace {
int getattr(const char *path, struct stat *stbuf) {
  return FUSE_OBJ->getattr(path, stbuf);
}

int fgetattr(const char *path, struct stat *stbuf, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fgetattr(path, stbuf, fileinfo);
}

int readlink(const char *path, char *buf, size_t size) {
  return FUSE_OBJ->readlink(path, buf, size);
}

int mknod(const char *path, mode_t mode, dev_t rdev) {
  return FUSE_OBJ->mknod(path, mode, rdev);
}

int mkdir(const char *path, mode_t mode) {
  return FUSE_OBJ->mkdir(path, mode);
}

int unlink(const char *path) {
  return FUSE_OBJ->unlink(path);
}

int rmdir(const char *path) {
  return FUSE_OBJ->rmdir(path);
}

int symlink(const char *from, const char *to) {
  return FUSE_OBJ->symlink(from, to);
}

int rename(const char *from, const char *to) {
  return FUSE_OBJ->rename(from, to);
}

int link(const char *from, const char *to) {
  return FUSE_OBJ->link(from, to);
}

int chmod(const char *path, mode_t mode) {
  return FUSE_OBJ->chmod(path, mode);
}

int chown(const char *path, uid_t uid, gid_t gid) {
  return FUSE_OBJ->chown(path, uid, gid);
}

int truncate(const char *path, off_t size) {
  return FUSE_OBJ->truncate(path, size);
}

int ftruncate(const char *path, off_t size, fuse_file_info *fileinfo) {
  return FUSE_OBJ->ftruncate(path, size, fileinfo);
}

int utimens(const char *path, const timespec times[2]) {
  return FUSE_OBJ->utimens(path, times);
}

int open(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->open(path, fileinfo);
}

int release(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->release(path, fileinfo);
}

int read(const char *path, char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->read(path, buf, size, offset, fileinfo);
}

int write(const char *path, const char *buf, size_t size, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->write(path, buf, size, offset, fileinfo);
}

int statfs(const char *path, struct statvfs *fsstat) {
  return FUSE_OBJ->statfs(path, fsstat);
}

int flush(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->flush(path, fileinfo);
}

int fsync(const char *path, int flags, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsync(path, flags, fileinfo);
}

//int setxattr(const char*, const char*, const char*, size_t, int)
//int getxattr(const char*, const char*, char*, size_t)
//int listxattr(const char*, char*, size_t)
//int removexattr(const char*, const char*)

int opendir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->opendir(path, fileinfo);
}

int readdir(const char *path, void *buf, fuse_fill_dir_t filler, off_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->readdir(path, buf, filler, offset, fileinfo);
}

int releasedir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->releasedir(path, fileinfo);
}

int fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsyncdir(path, datasync, fileinfo);
}

void* init(fuse_conn_info *conn) {
  return FUSE_OBJ->init(conn);
}

void destroy(void *userdata) {
  return FUSE_OBJ->destroy(userdata);
}

int access(const char *path, int mask) {
  return FUSE_OBJ->access(path, mask);
}

int create(const char *path, mode_t mode, fuse_file_info *fileinfo) {
  return FUSE_OBJ->create(path, mode, fileinfo);
}

/*int lock(const char*, fuse_file_info*, int cmd, flock*)
int bmap(const char*, size_t blocksize, uint64_t *idx)
int ioctl(const char*, int cmd, void *arg, fuse_file_info*, unsigned int flags, void *data)
int poll(const char*, fuse_file_info*, fuse_pollhandle *ph, unsigned *reventsp)
int write_buf(const char*, fuse_bufvec *buf, off_t off, fuse_file_info*)
int read_buf(const chas*, struct fuse_bufvec **bufp, size_t size, off_T off, fuse_file_info*)
int flock(const char*, fuse_file_info*, int op)
int fallocate(const char*, int, off_t, off_t, fuse_file_info*)*/

fuse_operations *operations() {
  static unique_ptr<fuse_operations> singleton(nullptr);

  if (!singleton) {
    singleton = make_unique<fuse_operations>();
    singleton->getattr = &getattr;
    singleton->fgetattr = &fgetattr;
    singleton->readlink = &readlink;
    singleton->mknod = &mknod;
    singleton->mkdir = &mkdir;
    singleton->unlink = &unlink;
    singleton->rmdir = &rmdir;
    singleton->symlink = &symlink;
    singleton->rename = &rename;
    singleton->link = &link;
    singleton->chmod = &chmod;
    singleton->chown = &chown;
    singleton->truncate = &truncate;
    singleton->utimens = &utimens;
    singleton->open = &open;
    singleton->read = &read;
    singleton->write = &write;
    singleton->statfs = &statfs;
    singleton->flush = &flush;
    singleton->release = &release;
    singleton->fsync = &fsync;
  /*#ifdef HAVE_SYS_XATTR_H
    singleton->setxattr = &setxattr;
    singleton->getxattr = &getxattr;
    singleton->listxattr = &listxattr;
    singleton->removexattr = &removexattr;
  #endif*/
    singleton->opendir = &opendir;
    singleton->readdir = &readdir;
    singleton->releasedir = &releasedir;
    singleton->fsyncdir = &fsyncdir;
    singleton->init = &init;
    singleton->destroy = &destroy;
    singleton->access = &access;
    singleton->create = &create;
    singleton->ftruncate = &ftruncate;
  }

  return singleton.get();
}
}

Fuse::~Fuse() {
}

void Fuse::run(int argc, char **argv) {
  fuse_main(argc, argv, operations(), (void*)this);
}
