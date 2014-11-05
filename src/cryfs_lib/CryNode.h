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

  void stat(struct stat *result) const;

protected:
  bf::path base_path() const;
  CryDevice *device();

private:
  CryDevice *const _device;
  const bf::path _path;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

inline bf::path CryNode::base_path() const {
  return _device->RootDir() / _path;
}

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYNODE_H_ */
