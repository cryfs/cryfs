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
  CryNode(CryDevice *device, boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> parent, const blockstore::Key &key);
  void access(int mask) const override;
  void stat(struct ::stat *result) const override;
  void chmod(mode_t mode) override;
  void chown(uid_t uid, gid_t gid) override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(timespec lastAccessTime, timespec lastModificationTime) override;

protected:
  CryNode();
  virtual ~CryNode();

  CryDevice *device();
  const CryDevice *device() const;
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob() const;
  std::shared_ptr<const parallelaccessfsblobstore::DirBlobRef> parent() const;

  virtual fspp::Dir::EntryType getType() const = 0;

  void removeNode();

private:
  CryDevice *_device;
  boost::optional<std::shared_ptr<parallelaccessfsblobstore::DirBlobRef>> _parent;
  blockstore::Key _key;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
