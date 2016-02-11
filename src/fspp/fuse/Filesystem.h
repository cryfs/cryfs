#pragma once
#ifndef MESSMER_FSPP_FUSE_FILESYSTEM_H_
#define MESSMER_FSPP_FUSE_FILESYSTEM_H_

#include <boost/filesystem.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include "../fs_interface/Dir.h"

namespace fspp {
namespace fuse {
class Filesystem {
public:
  virtual ~Filesystem() {}

  //TODO Test uid/gid parameters of createAndOpenFile
  virtual int createAndOpenFile(const boost::filesystem::path &path, mode_t mode, uid_t uid, gid_t gid) = 0;
  virtual int openFile(const boost::filesystem::path &path, int flags) = 0;
  virtual void flush(int descriptor) = 0;
  virtual void closeFile(int descriptor) = 0;
  virtual void lstat(const boost::filesystem::path &path, struct ::stat *stbuf) = 0;
  virtual void fstat(int descriptor, struct ::stat *stbuf) = 0;
  //TODO Test chmod
  virtual void chmod(const boost::filesystem::path &path, mode_t mode) = 0;
  //TODO Test chown
  virtual void chown(const boost::filesystem::path &path, uid_t uid, gid_t gid) = 0;
  virtual void truncate(const boost::filesystem::path &path, off_t size) = 0;
  virtual void ftruncate(int descriptor, off_t size) = 0;
  virtual size_t read(int descriptor, void *buf, size_t count, off_t offset) = 0;
  virtual void write(int descriptor, const void *buf, size_t count, off_t offset) = 0;
  virtual void fsync(int descriptor) = 0;
  virtual void fdatasync(int descriptor) = 0;
  virtual void access(const boost::filesystem::path &path, int mask) = 0;
  //TODO Test uid/gid parameters of mkdir
  virtual void mkdir(const boost::filesystem::path &path, mode_t mode, uid_t uid, gid_t gid) = 0;
  virtual void rmdir(const boost::filesystem::path &path) = 0;
  virtual void unlink(const boost::filesystem::path &path) = 0;
  virtual void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) = 0;
  virtual void utimens(const boost::filesystem::path &path, timespec lastAccessTime, timespec lastModificationTime) = 0;
  virtual void statfs(const boost::filesystem::path &path, struct statvfs *fsstat) = 0;
  //TODO We shouldn't use Dir::Entry here, that's in another layer
  virtual cpputils::unique_ref<std::vector<Dir::Entry>> readDir(const boost::filesystem::path &path) = 0;
  //TODO Test createSymlink
  virtual void createSymlink(const boost::filesystem::path &to, const boost::filesystem::path &from, uid_t uid, gid_t gid) = 0;
  //TODO Test readSymlink
  virtual void readSymlink(const boost::filesystem::path &path, char *buf, size_t size) = 0;
};

}
}

#endif
