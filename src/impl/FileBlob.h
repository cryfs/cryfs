#pragma once
#ifndef CRYFS_LIB_IMPL_FILEBLOB_H_
#define CRYFS_LIB_IMPL_FILEBLOB_H_

#include <messmer/blobstore/interface/Blob.h>
#include <memory>

namespace cryfs {

class FileBlob {
public:
  static std::unique_ptr<FileBlob> InitializeEmptyFile(std::unique_ptr<blobstore::Blob> blob);

  FileBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~FileBlob();

  ssize_t read(void *target, uint64_t offset, uint64_t count) const;
  void write(const void *source, uint64_t offset, uint64_t count);
  void flush();

  void resize(off_t size);
  off_t size() const;

  blockstore::Key key() const;

private:
  std::unique_ptr<blobstore::Blob> _blob;

  unsigned char magicNumber() const;
};

}

#endif
