#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTOREWITHRANDOMKEYS_H_
#define FSPP_BLOBSTORE_BLOBSTOREWITHRANDOMKEYS_H_

#include "blobstore/interface/Blob.h"
#include "blobstore/interface/BlobStore.h"

namespace blobstore {

// This is an implementation helpers for BlobStores that use random blob keys.
// You should never give this static type to the client. The client should always
// work with the BlobStore interface instead.
class BlobStoreWithRandomKeys: public BlobStore {
public:
  // Return nullptr if key already exists
  virtual std::unique_ptr<BlobWithKey> create(const std::string &key, size_t size) = 0;

  BlobWithKey create(size_t size) final;

private:
  std::string _generateRandomKey();
};

}

#endif
