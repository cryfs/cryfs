#pragma once
#ifndef FSPP_IMPL_FILESYSTEM_H_
#define FSPP_IMPL_FILESYSTEM_H_

#include <boost/filesystem.hpp>
#include <memory>
#include <sys/stat.h>
#include <sys/statvfs.h>

namespace fspp {
namespace fuse {
class Filesystem {
public:
  virtual ~Filesystem() {}

  virtual int createAndOpenFile(const boost::filesystem::path &path, mode_t mode) = 0;
  virtual int openFile(const boost::filesystem::path &path, int flags) = 0;
  virtual void flush(int descriptor) = 0;
  virtual void closeFile(int descriptor) = 0;
  virtual void lstat(const boost::filesystem::path &path, struct ::stat *stbuf) = 0;
  virtual void fstat(int descriptor, struct ::stat *stbuf) = 0;
  virtual void truncate(const boost::filesystem::path &path, off_t size) = 0;
  virtual void ftruncate(int descriptor, off_t size) = 0;
  virtual int read(int descriptor, void *buf, size_t count, off_t offset) = 0;
  virtual void write(int descriptor, const void *buf, size_t count, off_t offset) = 0;
  virtual void fsync(int descriptor) = 0;
  virtual void fdatasync(int descriptor) = 0;
  virtual void access(const boost::filesystem::path &path, int mask) = 0;
  virtual void mkdir(const boost::filesystem::path &path, mode_t mode) = 0;
  virtual void rmdir(const boost::filesystem::path &path) = 0;
  virtual void unlink(const boost::filesystem::path &path) = 0;
  virtual void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) = 0;
  virtual void utimens(const boost::filesystem::path &path, const timespec times[2]) = 0;
  virtual void statfs(const boost::filesystem::path &path, struct statvfs *fsstat) = 0;
  virtual std::unique_ptr<std::vector<std::string>> readDir(const boost::filesystem::path &path) = 0;
};

}
}

#endif
