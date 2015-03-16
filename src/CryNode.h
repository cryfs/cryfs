#pragma once
#ifndef CRYFS_LIB_CRYNODE_H_
#define CRYFS_LIB_CRYNODE_H_

#include <messmer/fspp/fs_interface/Node.h>
#include "messmer/cpp-utils/macros.h"

#include "CryDevice.h"

namespace cryfs {

class CryNode: public virtual fspp::Node {
public:
  void access(int mask) const override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(const timespec times[2]) override;

protected:
  CryNode();
  virtual ~CryNode();

private:

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
