#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_

#include <fspp/fs_interface/OpenFile.h>
#include "parallelaccessfsblobstore/FileBlobRef.h"
#include "parallelaccessfsblobstore/DirBlobRef.h"

namespace cryfs {
class CryDevice;

class CryOpenFile final: public fspp::OpenFile {
public:
  explicit CryOpenFile(const CryDevice *device, std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> parent, cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> fileBlob);
  ~CryOpenFile();

  void stat(struct ::stat *result) const override;
  void truncate(off_t size) const override;
  size_t read(void *buf, size_t count, off_t offset) const override;
  void write(const void *buf, size_t count, off_t offset) override;
  void flush() override;
  void fsync() override;
  void fdatasync() override;

private:
  const CryDevice *_device;
  std::shared_ptr<parallelaccessfsblobstore::DirBlobRef> _parent;
  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> _fileBlob;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
