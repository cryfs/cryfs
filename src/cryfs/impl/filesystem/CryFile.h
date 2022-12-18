#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_

#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FileBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include <fspp/fs_interface/File.h>
#include "CryNode.h"

namespace cryfs {

class CryFile final: public fspp::File, public CryNode {
public:
  CryFile(CryDevice *device, cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> parent, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> grandparent, const blockstore::BlockId &blockId);
  ~CryFile();

  cpputils::unique_ref<fspp::OpenFile> open(fspp::openflags_t flags) override;
  void truncate(fspp::num_bytes_t size) override;
  fspp::Dir::EntryType getType() const override;
  void remove() override;

private:
  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

}

#endif
