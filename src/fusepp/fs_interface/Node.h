#pragma once
#ifndef FUSEPP_NODE_H_
#define FUSEPP_NODE_H_

#include <boost/filesystem.hpp>

#include <sys/stat.h>

namespace fusepp {

class Node {
public:
  virtual ~Node() {}

  virtual void stat(struct ::stat *result) const = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path &to) = 0;
  virtual void utimens(const timespec times[2]) = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_NODE_H_ */
