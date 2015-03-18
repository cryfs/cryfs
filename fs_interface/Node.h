#pragma once
#ifndef FSPP_NODE_H_
#define FSPP_NODE_H_

#include <boost/filesystem.hpp>

#include <sys/stat.h>

namespace fspp {

class Node {
public:
  virtual ~Node() {}

  virtual void stat(struct ::stat *result) const = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path &to) = 0;
  virtual void utimens(const timespec times[2]) = 0;
};

}

#endif
