#pragma once
#ifndef FUSEPP_FUSEDIR_H_
#define FUSEPP_FUSEDIR_H_

#include "FuseNode.h"
#include <memory>
#include <string>

namespace fusepp {
class FuseDevice;
class FuseFile;

class FuseDir: public virtual FuseNode {
public:
  virtual ~FuseDir() {}

  virtual std::unique_ptr<FuseFile> createFile(const std::string &name, mode_t mode) = 0;
  virtual std::unique_ptr<FuseDir> createDir(const std::string &name, mode_t mode) = 0;
  virtual void rmdir() = 0;

  virtual std::unique_ptr<std::vector<std::string>> children() const = 0;
};

} /* namespace fusepp */

#endif /* FUSEPP_FUSEDIR_H_ */
