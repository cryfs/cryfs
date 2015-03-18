#pragma once
#ifndef CRYFS_LIB_IMPL_DIRBLOB_H_
#define CRYFS_LIB_IMPL_DIRBLOB_H_

#include <messmer/blobstore/interface/Blob.h>
#include <messmer/blockstore/utils/Key.h>
#include "messmer/cpp-utils/macros.h"
#include <messmer/fspp/fs_interface/Dir.h>

#include <memory>
#include <vector>

namespace cryfs{

class DirBlob {
public:
  static std::unique_ptr<DirBlob> InitializeEmptyDir(std::unique_ptr<blobstore::Blob> blob);

  DirBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~DirBlob();

  std::unique_ptr<std::vector<fspp::Dir::Entry>> GetChildren() const;
  //TODO Use struct instead of pair
  std::pair<fspp::Dir::EntryType, blockstore::Key> GetChild(const std::string &name) const;
  void AddChildDir(const std::string &name, const blockstore::Key &blobKey);
  void AddChildFile(const std::string &name, const blockstore::Key &blobKey);

private:
  unsigned char magicNumber() const;

  void AddChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type);

  const char *readAndAddNextChild(const char *pos, std::vector<fspp::Dir::Entry> *result) const;
  const char *getStartingPosOfEntry(const char *pos, const std::string &name) const;
  bool hasChild(const std::string &name) const;

  std::unique_ptr<blobstore::Blob> _blob;

  DISALLOW_COPY_AND_ASSIGN(DirBlob);
};

}

#endif
