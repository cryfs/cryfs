#pragma once
#ifndef FUSEPP_FUSEFILE_H_
#define FUSEPP_FUSEFILE_H_

#include "FuseNode.h"
#include <memory>

namespace fusepp {
class FuseDevice;
class FuseOpenFile;

class FuseFile: public virtual FuseNode {
public:
  virtual ~FuseFile() {}

  virtual std::unique_ptr<FuseOpenFile> open(int flags) const = 0;
  virtual void truncate(off_t size) const = 0;
  virtual void unlink() = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_FUSEFILE_H_ */
