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
  explicit CryOpenFile(const CryDevice *device, std::shared_ptr<fsblobstore::DirBlob> parent, cpputils::unique_ref<fsblobstore::FileBlob> fileBlob);
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
  std::shared_ptr<fsblobstore::DirBlob> _parent;
  cpputils::unique_ref<fsblobstore::FileBlob> _fileBlob;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
