#pragma once
#ifndef FUSEPP_FUSENODE_H_
#define FUSEPP_FUSENODE_H_

#include <boost/filesystem.hpp>
#include <fusepp/FuseDevice.h>

#include "utils/macros.h"
#include <sys/stat.h>

namespace fusepp {

namespace bf = boost::filesystem;

class FuseNode {
public:
  FuseNode(FuseDevice *device, const bf::path &path);
  virtual ~FuseNode();

  void stat(struct ::stat *result) const;
  void access(int mask) const;
  void rename(const bf::path &to);
  void utimens(const timespec times[2]);

protected:
  bf::path base_path() const;
  const bf::path &path() const;
  FuseDevice *device();
  const FuseDevice *device() const;

private:
  FuseDevice *const _device;
  bf::path _path;

  DISALLOW_COPY_AND_ASSIGN(FuseNode);
};

inline bf::path FuseNode::base_path() const {
  return _device->RootDir() / _path;
}

inline const bf::path &FuseNode::path() const {
  return _path;
}

inline FuseDevice *FuseNode::device() {
  return const_cast<FuseDevice*>(const_cast<const FuseNode*>(this)->device());
}

inline const FuseDevice *FuseNode::device() const {
  return _device;
}

} /* namespace fusepp */

#endif /* FUSEPP_FUSENODE_H_ */
