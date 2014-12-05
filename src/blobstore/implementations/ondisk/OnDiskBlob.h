#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_

#include "blobstore/interface/Blob.h"
#include "Data.h"

#include <boost/filesystem/path.hpp>
#include <iostream>

namespace blobstore {
namespace ondisk {
class OnDiskBlobStore;

class OnDiskBlob: public Blob {
public:
  OnDiskBlob(const boost::filesystem::path &filepath, size_t size);
  virtual ~OnDiskBlob();

  static std::unique_ptr<OnDiskBlob> LoadFromDisk(const boost::filesystem::path &filepath);
  static std::unique_ptr<OnDiskBlob> CreateOnDisk(const boost::filesystem::path &filepath, size_t size);

  void *data() override;
  const void *data() const override;

  size_t size() const override;

private:
  const boost::filesystem::path _filepath;
  size_t _size;
  Data _data;

  static void _assertFileDoesntExist(const boost::filesystem::path &filepath);
  static size_t _getStreamSize(std::istream &stream);
  void _loadDataFromStream(std::istream &stream);
  void _storeDataToStream(std::ostream &stream) const;
  void _fillDataWithZeroes();
  void _storeToDisk() const;
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
