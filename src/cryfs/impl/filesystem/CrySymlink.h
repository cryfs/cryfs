#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYSYMLINK_H_

#include <fspp/fs_interface/Symlink.h>
#include "CryNode.h"
#include "parallelaccessfsblobstore/SymlinkBlobRef.h"
#include "parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {

class CrySymlink final: public fspp::Symlink, CryNode {
public:
  CrySymlink(CryDevice *device, cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> parent, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> grandparent, const blockstore::Key &key);
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
