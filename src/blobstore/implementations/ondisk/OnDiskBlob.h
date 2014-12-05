#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOB_H_

#include "blobstore/interface/Blob.h"
#include "Data.h"

#include <iostream>

namespace blobstore {
namespace ondisk {
class OnDiskBlobStore;

class OnDiskBlob: public Blob {
public:
  OnDiskBlob(size_t size);
  virtual ~OnDiskBlob();

  void *data() override;
  const void *data() const override;

  size_t size() const override;

  void LoadDataFromStream(std::istream &stream);
  void StoreDataToStream(std::ostream &stream) const;
  void FillDataWithZeroes();

private:
  size_t _size;
  Data _data;
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
