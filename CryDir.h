#pragma once
#ifndef CRYFS_LIB_CRYDIR_H_
#define CRYFS_LIB_CRYDIR_H_

#include <messmer/fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "impl/DirBlock.h"

namespace cryfs {

class CryDir: public fspp::Dir, CryNode {
public:
  CryDir(CryDevice *device, std::unique_ptr<DirBlock> block);
  virtual ~CryDir();

  //TODO return type variance to CryFile/CryDir?
  std::unique_ptr<fspp::File> createFile(const std::string &name, mode_t mode) override;
  std::unique_ptr<fspp::Dir> createDir(const std::string &name, mode_t mode) override;
  void rmdir() override;

  std::unique_ptr<std::vector<std::string>> children() const override;

private:
  CryDevice *_device;
  std::unique_ptr<DirBlock> _block;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
