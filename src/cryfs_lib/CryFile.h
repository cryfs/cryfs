#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include <fspp/fs_interface/File.h>
#include "CryNode.h"

#include "impl/FileBlob.h"

namespace cryfs {

class CryFile: public fspp::File, CryNode {
public:
  CryFile(std::unique_ptr<FileBlob> blob);
  virtual ~CryFile();

  std::unique_ptr<fspp::OpenFile> open(int flags) const override;
  void truncate(off_t size) const override;
  void unlink() override;

private:
  std::unique_ptr<FileBlob> _blob;

  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

}

#endif
