#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_

#include <fspp/fs_interface/OpenFile.h>
#include <memory>
#include <mutex>
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FileBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {
class CryDevice;

class CryOpenFile final: public fspp::OpenFile {
public:
  explicit CryOpenFile(CryDevice *device, std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> parent, cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> fileBlob);
  ~CryOpenFile() override;

  stat_info stat() const override;
  void truncate(fspp::num_bytes_t size) const override;
  fspp::num_bytes_t read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const override;
  void write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) override;
  void flush() override;
  void fsync() override;
  void fdatasync() override;
  fspp::TimestampUpdateBehavior timestampUpdateBehavior() const;

private:
  // The file can be moved into a different directory while it is open. Our dir entry (which stores
  // for example the timestamps) then lives in the new parent directory blob and the parent we
  // remembered when the file was opened is stale. This returns the directory blob that currently
  // holds our dir entry, reloading it if the file was moved.
  std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> _parentBlob() const;

  CryDevice *_device;
  mutable std::mutex _parentMutex;
  mutable std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> _parent;
  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> _fileBlob;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
