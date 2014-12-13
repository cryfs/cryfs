#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTORE_H_
#define FSPP_BLOBSTORE_BLOBSTORE_H_

#include "Blob.h"
#include <string>
#include <memory>

#include "blockstore/utils/Key.h"

namespace blobstore {

class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual std::unique_ptr<Blob> create() = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual std::unique_ptr<Blob> load(const blockstore::Key &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blocks enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
