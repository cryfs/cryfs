#pragma once
#ifndef CRYFS_LIB_CRYNODE_H_
#define CRYFS_LIB_CRYNODE_H_

#include "fusepp/FuseNode.h"
#include "fusepp/utils/macros.h"

#include "CryDevice.h"

namespace cryfs {

class CryNode: public virtual fusepp::FuseNode {
public:
  CryNode(CryDevice *device, const boost::filesystem::path &path);
  virtual ~CryNode();

  void stat(struct ::stat *result) const override;
  void access(int mask) const override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(const timespec times[2]) override;

protected:
  boost::filesystem::path base_path() const;
  const boost::filesystem::path &path() const;
  CryDevice *device();
  const CryDevice *device() const;

private:
  CryDevice *const _device;
  boost::filesystem::path _path;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

inline boost::filesystem::path CryNode::base_path() const {
  return _device->RootDir() / _path;
}

inline const boost::filesystem::path &CryNode::path() const {
  return _path;
}

inline CryDevice *CryNode::device() {
  return const_cast<CryDevice*>(const_cast<const CryNode*>(this)->device());
}

inline const CryDevice *CryNode::device() const {
  return _device;
}

} /* namespace cryfs */

#endif
