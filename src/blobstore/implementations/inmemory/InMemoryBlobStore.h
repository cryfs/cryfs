#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOBSTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOBSTORE_H_

#include "blobstore/interface/helpers/BlobStoreWithRandomKeys.h"

#include "fspp/utils/macros.h"

#include <mutex>
#include <map>

namespace blobstore {
namespace inmemory {
class InMemoryBlob;

class InMemoryBlobStore: public BlobStoreWithRandomKeys {
public:
  InMemoryBlobStore();

  std::unique_ptr<BlobWithKey> create(const std::string &key, size_t size) override;
  bool exists(const std::string &key) override;
  std::unique_ptr<Blob> load(const std::string &key) override;

private:
  std::map<std::string, InMemoryBlob> _blobs;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlobStore);
};

} /* namespace inmemory */
} /* namespace blobstore */

#endif
