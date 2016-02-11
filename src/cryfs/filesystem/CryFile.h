#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_

#include "parallelaccessfsblobstore/FileBlobRef.h"
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include <fspp/fs_interface/File.h>
#include "CryNode.h"

namespace cryfs {

class CryFile final: public fspp::File, CryNode {
public:
  CryFile(CryDevice *device, cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> parent, const blockstore::Key &key);
  ~CryFile();

  cpputils::unique_ref<fspp::OpenFile> open(int flags) const override;
  void truncate(off_t size) const override;
  fspp::Dir::EntryType getType() const override;

private:
  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

}

#endif
