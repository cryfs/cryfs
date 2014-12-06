#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTOREWITHRANDOMKEYS_H_
#define FSPP_BLOBSTORE_BLOBSTOREWITHRANDOMKEYS_H_

#include <mutex>

#include "blobstore/interface/Blob.h"
#include "blobstore/interface/BlobStore.h"

namespace blobstore {

class BlobStoreWithRandomKeys: public BlobStore {
public:
  BlobStoreWithRandomKeys();
  virtual ~BlobStoreWithRandomKeys() {}

  BlobWithKey create(size_t size) override;

  virtual std::unique_ptr<Blob> load(const std::string &key) = 0;

protected:
  virtual BlobWithKey create(const std::string &key, size_t size) = 0;

private:
  std::string _generateKey();
  std::string _generateRandomKey();

  std::mutex _generate_key_mutex;
};

}

#endif
