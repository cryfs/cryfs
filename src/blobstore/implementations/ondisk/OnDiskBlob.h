#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_

#include "blobstore/interface/Blob.h"
#include "blobstore/utils/Data.h"

#include <boost/filesystem/path.hpp>
#include <iostream>

#include "fspp/utils/macros.h"

namespace blobstore {
namespace ondisk {
class OnDiskBlobStore;

class OnDiskBlob: public Blob {
public:
  virtual ~OnDiskBlob();

  static std::unique_ptr<OnDiskBlob> LoadFromDisk(const boost::filesystem::path &filepath);
  static std::unique_ptr<OnDiskBlob> CreateOnDisk(const boost::filesystem::path &filepath, size_t size);

  void *data() override;
  const void *data() const override;

  void flush() override;

  size_t size() const override;

private:
  const boost::filesystem::path _filepath;
  Data _data;

  OnDiskBlob(const boost::filesystem::path &filepath, size_t size);
  OnDiskBlob(const boost::filesystem::path &filepath, Data &&data);

  static void _assertFileDoesntExist(const boost::filesystem::path &filepath);
  void _fillDataWithZeroes();
  void _storeToDisk() const;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlob);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
