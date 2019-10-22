#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_NODE_H_
#define MESSMER_FSPP_FSINTERFACE_NODE_H_

#include <boost/filesystem.hpp>
#include "Types.h"
#include "Dir.h"

#include <blockstore/utils/BlockId.h>


namespace fspp {


class Node {
public:
  virtual ~Node() {}

  using stat_info = fspp::stat_info;

  virtual stat_info stat() const = 0;
  virtual void chmod(fspp::mode_t mode) = 0;
  virtual void chown(fspp::uid_t uid, fspp::gid_t gid) = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path& from, const boost::filesystem::path &to) = 0; // 'to' will always be an absolute path (but on Windows without the device specifier, i.e. starting with '/')
  virtual void utimens(const timespec lastAccessTime, const timespec lastModificationTime) = 0;
  virtual void remove() = 0;
  virtual void link() = 0;
  virtual bool unlink() = 0; // return true iff the last link was removed
  virtual Dir::NodeType getType() const = 0;

  virtual const blockstore::BlockId& blockId() const = 0;


};

}

#endif
