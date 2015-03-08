#pragma once
#ifndef CRYFS_LIB_IMPL_FILEBLOB_H_
#define CRYFS_LIB_IMPL_FILEBLOB_H_

#include <messmer/blobstore/interface/Blob.h>
#include <memory>

namespace cryfs {

class FileBlob {
public:
  FileBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~FileBlob();

  static bool IsFile(const blobstore::Blob &blob);

  void InitializeEmptyFile();

  void read(void *target, uint64_t offset, uint64_t count) const;
  void write(const void *source, uint64_t offset, uint64_t count);

  blockstore::Key key() const;

private:
  std::unique_ptr<blobstore::Blob> _blob;

  unsigned char magicNumber() const;
  static unsigned char magicNumber(const blobstore::Blob &blob);
};

}

#endif
