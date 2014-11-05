#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include <memory>

#include "CryDevice.h"
#include "CryNode.h"

namespace cryfs {

class CryFile: public CryNode {
public:
  CryFile(CryDevice *device, const bf::path &path);
  virtual ~CryFile();
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFILE_H_ */
