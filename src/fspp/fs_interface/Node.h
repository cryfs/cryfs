#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_NODE_H_
#define MESSMER_FSPP_FSINTERFACE_NODE_H_

#include <boost/filesystem.hpp>

#include <sys/stat.h>

namespace fspp {

class Node {
public:
  virtual ~Node() {}

  virtual void stat(struct ::stat *result) const = 0;
  virtual void chmod(mode_t mode) = 0;
  virtual void chown(uid_t uid, gid_t gid) = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path &to) = 0; // 'to' will always be an absolute path
  virtual void utimens(const timespec lastAccessTime, const timespec lastModificationTime) = 0;
  virtual void remove() = 0;
};

}

#endif
