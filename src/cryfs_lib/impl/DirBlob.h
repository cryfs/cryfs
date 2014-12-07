#pragma once
#ifndef CRYFS_LIB_IMPL_DIRBLOB_H_
#define CRYFS_LIB_IMPL_DIRBLOB_H_

#include "blobstore/interface/Blob.h"
#include "fspp/utils/macros.h"

#include <memory>
#include <vector>

namespace cryfs{

class DirBlob {
public:
  DirBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~DirBlob();

  void InitializeEmptyDir();
  std::unique_ptr<std::vector<std::string>> GetChildren() const;
  void AddChild(const std::string &name, const std::string &blobKey);
  std::string GetBlobKeyForName(const std::string &name) const;

private:
  unsigned int *entryCounter();
  const unsigned int *entryCounter() const;
  char *entriesBegin();
  const char *entriesBegin() const;
  char *entriesEnd();

  const char *readAndAddNextChild(const char *pos, std::vector<std::string> *result) const;
  void assertEnoughSpaceLeft(char *insertPos, size_t insertSize) const;

  std::unique_ptr<blobstore::Blob> _blob;

  DISALLOW_COPY_AND_ASSIGN(DirBlob);
};

}

#endif
