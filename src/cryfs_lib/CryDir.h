#pragma once
#ifndef CRYFS_LIB_CRYDIR_H_
#define CRYFS_LIB_CRYDIR_H_

#include "CryNode.h"
#include "utils/macros.h"

namespace cryfs {
class CryDevice;

class CryDir: public CryNode {
public:
  CryDir(CryDevice *device, const bf::path &path);
  virtual ~CryDir();
private:
  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYDIR_H_ */
