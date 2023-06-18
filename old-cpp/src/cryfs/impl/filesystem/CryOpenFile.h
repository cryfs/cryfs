#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_

#include <fspp/fs_interface/OpenFile.h>
#include "cryfs/impl/filesystem/rustfsblobstore/RustFileBlob.h"
#include "cryfs/impl/filesystem/rustfsblobstore/RustDirBlob.h"

namespace cryfs {
class CryDevice;

class CryOpenFile final: public fspp::OpenFile {
public:
  explicit CryOpenFile(CryDevice *device, const blockstore::BlockId &parentBlobId, const blockstore::BlockId& fileBlobId);
  ~CryOpenFile();

  stat_info stat() const override;
  void truncate(fspp::num_bytes_t size) const override;
  fspp::num_bytes_t read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const override;
  void write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) override;
  void flush() override;
  void fsync() override;
  void fdatasync() override;
  fspp::TimestampUpdateBehavior timestampUpdateBehavior() const;

private:
  cpputils::unique_ref<fsblobstore::rust::RustFileBlob> LoadFileBlob() const;
  cpputils::unique_ref<fsblobstore::rust::RustDirBlob> LoadParentBlob() const;

  CryDevice *_device;

  // We're storing the blob ids instead of the blobs themselves
  // because CryOpenFile instances are being kept in memory
  // for as long as the file is open and they would keep a lock
  // on the parent and file blob otherwise that would block other
  // operations.
  blockstore::BlockId _parentBlobId;
  blockstore::BlockId _fileBlobId;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
