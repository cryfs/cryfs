#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTORE_H_
#define FSPP_BLOBSTORE_BLOBSTORE_H_

#include <string>
#include <memory>

#include "Blob.h"
#include "blobstore/utils/BlobWithKey.h"

namespace blobstore {

//TODO Don't use string, but own class for keys? (better performance for all keys have same length)

class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual BlobWithKey create(size_t size) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if blob with this key doesn't exists
  virtual std::unique_ptr<Blob> load(const std::string &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blobs enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
