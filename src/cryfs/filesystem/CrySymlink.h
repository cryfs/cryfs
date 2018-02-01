#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_

#include <fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "fsblobstore/SymlinkBlob.h"
#include "fsblobstore/DirBlob.h"

namespace cryfs {

class CrySymlink final: public fspp::Symlink, public CryNode {
public:
  CrySymlink(CryDevice *device, boost::filesystem::path path, std::shared_ptr<fsblobstore::DirBlob> parent, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> grandparent, const blockstore::BlockId &blockId);
  ~CrySymlink();

  boost::filesystem::path target() override;

  fspp::Dir::EntryType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<fsblobstore::SymlinkBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CrySymlink);
};

}

#endif
