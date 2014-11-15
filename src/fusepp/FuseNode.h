#pragma once
#ifndef FUSEPP_FUSENODE_H_
#define FUSEPP_FUSENODE_H_

#include <boost/filesystem.hpp>

#include "utils/macros.h"
#include <sys/stat.h>

namespace fusepp {

class FuseNode {
public:
  virtual ~FuseNode() {}

  virtual void stat(struct ::stat *result) const = 0;
  virtual void access(int mask) const = 0;
  virtual void rename(const boost::filesystem::path &to) = 0;
  virtual void utimens(const timespec times[2]) = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_FUSENODE_H_ */
