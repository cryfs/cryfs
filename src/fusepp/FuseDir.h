#pragma once
#ifndef FUSEPP_FUSEDIR_H_
#define FUSEPP_FUSEDIR_H_

#include <fusepp/FuseNode.h>
#include <memory>
#include <string>

#include "utils/macros.h"

namespace fusepp {
class FuseDevice;

class FuseDir: public FuseNode {
public:
  FuseDir(FuseDevice *device, const bf::path &path);
  virtual ~FuseDir();

  std::unique_ptr<FuseFile> createFile(const std::string &name, mode_t mode);
  std::unique_ptr<FuseDir> createDir(const std::string &name, mode_t mode);
  void rmdir();

  std::unique_ptr<std::vector<std::string>> children() const;

private:
  DISALLOW_COPY_AND_ASSIGN(FuseDir);
};

} /* namespace fusepp */

#endif /* FUSEPP_FUSEDIR_H_ */
