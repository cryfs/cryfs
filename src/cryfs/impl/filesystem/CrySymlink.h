#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_

#include <fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/TimestampUpdateBehavior.h"

namespace cryfs {

class CrySymlink final: public fspp::Symlink, public CryNode {
public:
  CrySymlink(CryDevice *device, cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> parent, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> grandparent, const blockstore::BlockId &blockId, fsblobstore::TimestampUpdateBehavior timestampUpdateBehavior);
  ~CrySymlink();

  boost::filesystem::path target() override;

  fspp::Dir::EntryType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<parallelaccessfsblobstore::SymlinkBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CrySymlink);
};

}

#endif
