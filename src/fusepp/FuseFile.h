#pragma once
#ifndef FUSEPP_FUSEFILE_H_
#define FUSEPP_FUSEFILE_H_

#include <fusepp/FuseNode.h>
#include <memory>

#include "utils/macros.h"

namespace fusepp {
class FuseDevice;
class FuseOpenFile;

class FuseFile: public FuseNode {
public:
  FuseFile(FuseDevice *device, const bf::path &path);
  virtual ~FuseFile();

  std::unique_ptr<FuseOpenFile> open(int flags) const;
  void truncate(off_t size) const;
  void unlink();
private:
  DISALLOW_COPY_AND_ASSIGN(FuseFile);
};

} /* namespace fusepp */

#endif /* FUSEPP_FUSEFILE_H_ */
