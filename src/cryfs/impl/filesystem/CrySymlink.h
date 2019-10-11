#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_

#include <fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {

class CrySymlink final: public fspp::Symlink, public CryNode {
public:
  CrySymlink(CryDevice *device, const blockstore::BlockId &blockId);
  ~CrySymlink();

  boost::filesystem::path target() override;

  fspp::Dir::NodeType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<parallelaccessfsblobstore::SymlinkBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CrySymlink);
};

}

#endif
