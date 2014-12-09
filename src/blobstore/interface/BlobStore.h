#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTORE_H_
#define FSPP_BLOBSTORE_BLOBSTORE_H_

#include "Blob.h"
#include "blobstore/utils/BlobWithKey.h"
#include <string>
#include <memory>


namespace blobstore {

class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual BlobWithKey create(size_t size) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual std::unique_ptr<Blob> load(const std::string &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blocks enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
