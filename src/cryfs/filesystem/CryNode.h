#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_

#include <fspp/fs_interface/Node.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include "fsblobstore/DirBlob.h"
#include "CryDevice.h"

namespace cryfs {

class CryNode: public fspp::Node {
public:
  virtual ~CryNode();

  // TODO grandparent is only needed to set the timestamps of the parent directory on rename and remove. Delete grandparent parameter once we store timestamps in the blob itself instead of in the directory listing.
  CryNode(CryDevice *device, boost::filesystem::path path, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> parent, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> grandparent, const blockstore::BlockId &blockId);
  void access(int mask) const override;
  void stat(struct ::stat *result) const override;
  void chmod(mode_t mode) override;
  void chown(uid_t uid, gid_t gid) override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(timespec lastAccessTime, timespec lastModificationTime) override;

  // used in test cases
  bool checkParentPointer();

protected:
  CryNode();

  CryDevice *device();
  const CryDevice *device() const;
  const blockstore::BlockId &blockId() const;
  cpputils::unique_ref<fsblobstore::FsBlob> LoadBlob() const;
  bool isRootDir() const;
  std::shared_ptr<const fsblobstore::DirBlob> parent() const;
  std::shared_ptr<fsblobstore::DirBlob> parent();
  boost::optional<fsblobstore::DirBlob*> grandparent();

  virtual fspp::Dir::EntryType getType() const = 0;

  void removeNode();

private:
  void _updateParentModificationTimestamp();
  void _updateTargetDirModificationTimestamp(const fsblobstore::DirBlob &targetDir, boost::optional<std::shared_ptr<fsblobstore::DirBlob>> targetDirParent);

  CryDevice *_device;
  boost::filesystem::path _path;
  boost::optional<std::shared_ptr<fsblobstore::DirBlob>> _parent;
  boost::optional<std::shared_ptr<fsblobstore::DirBlob>> _grandparent;
  blockstore::BlockId _blockId;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
