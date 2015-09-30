#pragma once
#ifndef CRYFS_LIB_CRYSYMLINK_H_
#define CRYFS_LIB_CRYSYMLINK_H_

#include <messmer/fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "fsblobstore/SymlinkBlob.h"
#include "fsblobstore/DirBlob.h"

namespace cryfs {

class CrySymlink: public fspp::Symlink, CryNode {
public:
  CrySymlink(CryDevice *device, cpputils::unique_ref<fsblobstore::DirBlob> parent, const blockstore::Key &key);
  virtual ~CrySymlink();

  boost::filesystem::path target() const override;

  fspp::Dir::EntryType getType() const override;

private:
  cpputils::unique_ref<fsblobstore::SymlinkBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CrySymlink);
};

}

#endif
