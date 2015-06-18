#pragma once
#ifndef CRYFS_LIB_CRYDIR_H_
#define CRYFS_LIB_CRYDIR_H_

#include <messmer/fspp/fs_interface/Dir.h>
#include "CryNode.h"
#include "impl/DirBlob.h"

namespace cryfs {

class CryDir: public fspp::Dir, CryNode {
public:
  CryDir(CryDevice *device, boost::optional<cpputils::unique_ref<DirBlob>> parent, const blockstore::Key &key);
  virtual ~CryDir();

  //TODO return type variance to CryFile/CryDir?
  std::unique_ptr<fspp::OpenFile> createAndOpenFile(const std::string &name, mode_t mode, uid_t uid, gid_t gid) override;
  void createDir(const std::string &name, mode_t mode, uid_t uid, gid_t gid) override;
  void createSymlink(const std::string &name, const boost::filesystem::path &target, uid_t uid, gid_t gid) override;

  //TODO Make Entry a public class instead of hidden in DirBlob (which is not publicly visible)
  std::unique_ptr<std::vector<fspp::Dir::Entry>> children() const override;

  fspp::Dir::EntryType getType() const override;

private:
  boost::optional<cpputils::unique_ref<DirBlob>> LoadBlob() const;

  DISALLOW_COPY_AND_ASSIGN(CryDir);
};

}

#endif
