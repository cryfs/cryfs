#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_

#include <fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {

class CryDir final: public fspp::Dir, public CryNode {
public:
  CryDir(CryDevice *device, const blockstore::BlockId &blockId);
  ~CryDir();

  //TODO return type variance to CryFile/CryDir?
  cpputils::unique_ref<fspp::OpenFile> createAndOpenFile(const std::string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) override;
  void createDir(const std::string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) override;
  void createSymlink(const std::string &name, const boost::filesystem::path &target, fspp::uid_t uid, fspp::gid_t gid) override;
  void createLink(const boost::filesystem::path &target, const std::string& name);

  //TODO Make Entry a public class instead of hidden in DirBlob (which is not publicly visible)
  cpputils::unique_ref<std::vector<fspp::Dir::Entry>> children() override;

  fspp::Dir::NodeType getType() const override;

  void remove() override;

  void removeChildEntryByName(const string& name) override;
  void updateAccessTimestamp() override;
  void updateModificationTimestamp() override;
  void updateChangeTimestamp();

private:
  cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
