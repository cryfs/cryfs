#pragma once
#ifndef CRYFS_LIB_CRYNODE_H_
#define CRYFS_LIB_CRYNODE_H_

#include <boost/filesystem.hpp>

#include "utils/macros.h"
#include "CryDevice.h"
#include <sys/stat.h>

namespace cryfs {

namespace bf = boost::filesystem;

class CryNode {
public:
  CryNode(CryDevice *device, const bf::path &path);
  virtual ~CryNode();

  void stat(struct ::stat *result) const;
  void access(int mask) const;
  void rename(const bf::path &to);
  void utimens(const timespec times[2]);

protected:
  bf::path base_path() const;
  const bf::path &path() const;
  CryDevice *device();
  const CryDevice *device() const;

private:
  CryDevice *const _device;
  bf::path _path;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

inline bf::path CryNode::base_path() const {
  return _device->RootDir() / _path;
}

inline const bf::path &CryNode::path() const {
  return _path;
}

inline CryDevice *CryNode::device() {
  return const_cast<CryDevice*>(const_cast<const CryNode*>(this)->device());
}

inline const CryDevice *CryNode::device() const {
  return _device;
}

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYNODE_H_ */
