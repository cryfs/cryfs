#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_

#include <fspp/fs_interface/Node.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

namespace cryfs {

class CryDir;
class CryNode: public fspp::Node {
public:
  ~CryNode() override;

  CryNode(CryDevice *device, const blockstore::BlockId &blockId);
  void access(int mask) const override;
  stat_info stat() const override;
  void chmod(fspp::mode_t mode) override;
  void chown(fspp::uid_t uid, fspp::gid_t gid) override;
  void rename(const boost::filesystem::path& from, const boost::filesystem::path &to) override;
  void utimens(timespec lastAccessTime, timespec lastModificationTime) override;

  void link() override;
  bool unlink() override;

  fspp::Dir::EntryType getType() const override = 0;
  const blockstore::BlockId &blockId() const override;
protected:

  CryDevice *device();
  const CryDevice *device() const;
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob() const;
  bool isRootDir() const;

  virtual void updateChangeTimestamp();



  void removeNode();

private:

  CryDevice *_device;
  blockstore::BlockId _blockId;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
