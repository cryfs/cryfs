#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_NODE_H_
#define MESSMER_FSPP_FSINTERFACE_NODE_H_

#include <boost/filesystem.hpp>

namespace fspp {

class Node {
public:
  virtual ~Node() {}

  struct stat_info final {
      uint32_t nlink;
      uint32_t mode;
      uint32_t uid;
      uint32_t gid;
      uint64_t size;
      uint64_t blocks;
      struct timespec atime;
      struct timespec mtime;
      struct timespec ctime;
  };

  virtual stat_info stat() const = 0;
  virtual void chmod(mode_t mode) = 0;
  virtual void chown(uid_t uid, gid_t gid) = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path &to) = 0; // 'to' will always be an absolute path (but on Windows without the device specifier, i.e. starting with '/')
  virtual void utimens(const timespec lastAccessTime, const timespec lastModificationTime) = 0;
  virtual void remove() = 0;
};

}

#endif
