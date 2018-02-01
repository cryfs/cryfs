#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYFILE_H_

#include "fsblobstore/FileBlob.h"
#include "fsblobstore/DirBlob.h"
#include <fspp/fs_interface/File.h>
#include "CryNode.h"

namespace cryfs {

class CryFile final: public fspp::File, public CryNode {
public:
  CryFile(CryDevice *device, boost::filesystem::path path, std::shared_ptr<fsblobstore::DirBlob> parent, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> grandparent, const blockstore::BlockId &blockId);
  ~CryFile();

  cpputils::unique_ref<fspp::OpenFile> open(int flags) override;
  void truncate(off_t size) override;
  fspp::Dir::EntryType getType() const override;
  void remove() override;

private:
  cpputils::unique_ref<fsblobstore::FileBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryFile);
};

}

#endif
