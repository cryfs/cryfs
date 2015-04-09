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
  struct Entry {
    Entry(fspp::Dir::EntryType type_, const std::string &name_, const blockstore::Key &key_): type(type_), name(name_), key(key_) {}
    fspp::Dir::EntryType type;
    std::string name;
    blockstore::Key key;
  };

  static std::unique_ptr<DirBlob> InitializeEmptyDir(std::unique_ptr<blobstore::Blob> blob);

  DirBlob(std::unique_ptr<blobstore::Blob> blob);
  virtual ~DirBlob();

  void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;
  const Entry &GetChild(const std::string &name) const;
  void AddChildDir(const std::string &name, const blockstore::Key &blobKey);
  void AddChildFile(const std::string &name, const blockstore::Key &blobKey);
  void RemoveChild(const blockstore::Key &key);
  void flush();

private:
  unsigned char magicNumber() const;

  void AddChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type);

  const char *readAndAddNextChild(const char *pos, std::vector<Entry> *result) const;
  bool hasChild(const std::string &name) const;

  void _readEntriesFromBlob();
  void _writeEntriesToBlob();

  std::unique_ptr<blobstore::Blob> _blob;
  std::vector<Entry> _entries;
  bool _changed;

  DISALLOW_COPY_AND_ASSIGN(DirBlob);
};

}

#endif
