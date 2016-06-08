#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_

#include <fspp/fs_interface/Node.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

namespace cryfs {

class CryNode: public virtual fspp::Node {
public:
  virtual ~CryNode();

  // TODO grandparent is only needed to set the timestamps of the parent directory on rename and remove. Delete grandparent parameter once we store timestamps in the blob itself instead of in the directory listing.
  CryNode(CryDevice *device, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> parent, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> grandparent, const blockstore::Key &key);
  void access(int mask) const override;
  void stat(struct ::stat *result) const override;
  void chmod(mode_t mode) override;
  void chown(uid_t uid, gid_t gid) override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(timespec lastAccessTime, timespec lastModificationTime) override;

protected:
  CryNode();

  CryDevice *device();
  const CryDevice *device() const;
  const blockstore::Key &key() const;
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob() const;
  bool isRootDir() const;
  std::shared_ptr<const parallelaccessfsblobstore::DirBlobRef> parent() const;
  std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> parent();
  boost::optional<parallelaccessfsblobstore::DirBlobRef*> grandparent();

  virtual fspp::Dir::EntryType getType() const = 0;

  void removeNode();

private:
  void _updateParentModificationTimestamp();
  void _updateTargetDirModificationTimestamp(const parallelaccessfsblobstore::DirBlobRef &targetDir, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> targetDirParent);

  CryDevice *_device;
  boost::optional<std::shared_ptr<parallelaccessfsblobstore::DirBlobRef>> _parent;
  boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> _grandparent;
  blockstore::Key _key;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
