#pragma once
#ifndef CRYFS_LIB_CRYFILE_H_
#define CRYFS_LIB_CRYFILE_H_

#include "impl/FileBlob.h"
#include <messmer/fspp/fs_interface/File.h>
#include "CryNode.h"


namespace cryfs {

class CryFile: public fspp::File, CryNode {
public:
  CryFile(CryDevice *device, std::unique_ptr<DirBlob> parent, const blockstore::Key &key);
  virtual ~CryFile();

  std::unique_ptr<fspp::OpenFile> open(int flags) const override;
  void truncate(off_t size) const override;
  fspp::Dir::EntryType getType() const override;

private:

  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

}

#endif
