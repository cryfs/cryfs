#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_

#include <fspp/fs_interface/OpenFile.h>
#include "fsblobstore/FileBlob.h"
#include "fsblobstore/DirBlob.h"

namespace cryfs {
class CryDevice;

class CryOpenFile final: public fspp::OpenFile {
public:
  explicit CryOpenFile(CryDevice *device, blockstore::BlockId parentBlobId, blockstore::BlockId fileBlobId, std::weak_ptr<fsblobstore::DirBlob> parent);
  ~CryOpenFile();

  void stat(struct ::stat *result) const override;
  void truncate(off_t size) override;
  size_t read(void *buf, size_t count, off_t offset) const override;
  void write(const void *buf, size_t count, off_t offset) override;
  void flush() override;
  void fsync() override;
  void fdatasync() override;

private:
  cpputils::unique_ref<fsblobstore::FileBlob> _Load() const;
  std::shared_ptr<fsblobstore::DirBlob> _LoadParent() const;

  CryDevice *_device;
  blockstore::BlockId _parentBlobId;
  blockstore::BlockId _fileBlobId;
  // This weak_ptr is needed because the CryFile creating this CryOpenFile
  // stores a shared_ptr of the parent, and as long as that one's valid,
  // we have to use it instead of requesting our own (due to blocks being
  // locked in ThreadsafeBlockStore).
  std::weak_ptr<fsblobstore::DirBlob> _parent;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
