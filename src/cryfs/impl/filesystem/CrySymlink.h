#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_

#include <fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "cryfs/impl/filesystem/rustfsblobstore/RustSymlinkBlob.h"
#include "cryfs/impl/filesystem/rustfsblobstore/RustDirBlob.h"

namespace cryfs {

class CrySymlink final: public fspp::Symlink, public CryNode {
public:
  CrySymlink(CryDevice *device, cpputils::unique_ref<fsblobstore::rust::RustDirBlob> parent, boost::optional<cpputils::unique_ref<fsblobstore::rust::RustDirBlob>> grandparent, const blockstore::BlockId &blockId);
  ~CrySymlink();

  boost::filesystem::path target() override;

  fspp::Dir::EntryType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<fsblobstore::rust::RustSymlinkBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CrySymlink);
};

}

#endif
