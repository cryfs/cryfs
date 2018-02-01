#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_

#include <fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "fsblobstore/DirBlob.h"

namespace cryfs {

class CryDir final: public fspp::Dir, public CryNode {
public:
  CryDir(CryDevice *device, boost::filesystem::path path, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> parent, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> grandparent, const blockstore::BlockId &blockId);
  ~CryDir();

  //TODO return type variance to CryFile/CryDir?
  cpputils::unique_ref<fspp::OpenFile> createAndOpenFile(const std::string &name, mode_t mode, uid_t uid, gid_t gid) override;
  void createDir(const std::string &name, mode_t mode, uid_t uid, gid_t gid) override;
  void createSymlink(const std::string &name, const boost::filesystem::path &target, uid_t uid, gid_t gid) override;

  //TODO Make Entry a public class instead of hidden in DirBlob (which is not publicly visible)
  cpputils::unique_ref<std::vector<fspp::Dir::Entry>> children() override;

  fspp::Dir::EntryType getType() const override;

  void remove() override;

private:
  cpputils::unique_ref<fsblobstore::DirBlob> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
