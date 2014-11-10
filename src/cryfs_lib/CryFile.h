#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include <memory>

#include "CryDevice.h"
#include "CryNode.h"
#include "utils/macros.h"

namespace cryfs {
class CryOpenFile;

class CryFile: public CryNode {
public:
  CryFile(CryDevice *device, const bf::path &path);
  virtual ~CryFile();

  std::unique_ptr<CryOpenFile> open(int flags) const;
private:
  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFILE_H_ */
