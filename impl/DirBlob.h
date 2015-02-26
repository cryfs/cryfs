#pragma once
#ifndef CRYFS_LIB_IMPL_DIRBLOB_H_
#define CRYFS_LIB_IMPL_DIRBLOB_H_

#include <messmer/blobstore/interface/Blob.h>
#include <messmer/blockstore/utils/Key.h>
#include "messmer/cpp-utils/macros.h"

#include <memory>
#include <vector>

namespace cryfs{

class DirBlob {
public:
  DirBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~DirBlob();

  void InitializeEmptyDir();
  std::unique_ptr<std::vector<std::string>> GetChildren() const;
  void AddChild(const std::string &name, const blockstore::Key &blobKey);
  blockstore::Key GetBlobKeyForName(const std::string &name) const;

  static bool IsDir(const blobstore::Blob &blob);

private:
  unsigned char magicNumber() const;
  static const unsigned char magicNumber(const blobstore::Blob &blob);

  const char *readAndAddNextChild(const char *pos, std::vector<std::string> *result) const;

  std::unique_ptr<blobstore::Blob> _blob;

  DISALLOW_COPY_AND_ASSIGN(DirBlob);
};

}

#endif
