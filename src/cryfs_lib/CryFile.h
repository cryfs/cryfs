#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include "CryNode.h"
#include "fusepp/fs_interface/FuseFile.h"

namespace cryfs {

class CryFile: public fusepp::FuseFile, CryNode {
public:
  CryFile(CryDevice *device, const boost::filesystem::path &path);
  virtual ~CryFile();

  std::unique_ptr<fusepp::FuseOpenFile> open(int flags) const override;
  void truncate(off_t size) const override;
  void unlink() override;

private:
  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

} /* namespace cryfs */

#endif
