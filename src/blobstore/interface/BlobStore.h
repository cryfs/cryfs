#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTORE_H_
#define FSPP_BLOBSTORE_BLOBSTORE_H_

#include <string>
#include <memory>

#include "Blob.h"

namespace blobstore {

//TODO Don't use string, but own class for keys? (better performance for all keys have same length)

class BlobStore {
public:
  virtual ~BlobStore() {}

  struct BlobWithKey {
    BlobWithKey(const std::string &key_, std::unique_ptr<Blob> &&blob_): key(key_), blob(std::move(blob_)) {}

    std::string key;
    std::unique_ptr<Blob> blob;
  };

  virtual BlobWithKey create(size_t size) = 0;
  virtual std::unique_ptr<Blob> load(const std::string &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blobs enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
