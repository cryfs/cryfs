#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_

#include <fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {

class CryDir final: public fspp::Dir, public CryNode {
public:
  CryDir(CryDevice *device, boost::optional<blockstore::BlockId> parent, boost::optional<blockstore::BlockId> grandparent, const blockstore::BlockId &blockId);
  ~CryDir();

  //TODO return type variance to CryFile/CryDir?
  cpputils::unique_ref<fspp::OpenFile> createAndOpenFile(const std::string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) override;
  void createDir(const std::string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) override;
  void createSymlink(const std::string &name, const boost::filesystem::path &target, fspp::uid_t uid, fspp::gid_t gid) override;

  //TODO Make Entry a public class instead of hidden in DirBlob (which is not publicly visible)
  std::vector<fspp::Dir::Entry> children() override;
  size_t numChildren();

  fspp::Dir::EntryType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<fsblobstore::rust::RustDirBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
