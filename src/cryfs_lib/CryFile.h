#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include <memory>

#include "CryNode.h"
#include "utils/macros.h"

namespace cryfs {
class CryDevice;
class CryOpenFile;

class CryFile: public CryNode {
public:
  CryFile(CryDevice *device, const bf::path &path);
  virtual ~CryFile();

  std::unique_ptr<CryOpenFile> open(int flags) const;
  void truncate(off_t size) const;
  void unlink();
private:
  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYFILE_H_ */
