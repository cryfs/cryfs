#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDIR_H_

#include <fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {

class CryDir final: public fspp::Dir, CryNode {
public:
  CryDir(CryDevice *device, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> parent, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> grandparent, const blockstore::Key &key);
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
  cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
