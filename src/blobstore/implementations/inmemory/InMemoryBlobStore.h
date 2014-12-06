#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOBSTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOBSTORE_H_

#include "blobstore/interface/BlobStore.h"

#include "fspp/utils/macros.h"

#include <mutex>
#include <map>

namespace blobstore {
namespace inmemory {
class InMemoryBlob;

class InMemoryBlobStore: public BlobStore {
public:
  InMemoryBlobStore();

  BlobWithKey create(size_t size) override;
  std::unique_ptr<Blob> load(const std::string &key) override;

private:
  std::string _generateKey();
  std::string _generateRandomKey();

  std::map<std::string, InMemoryBlob> _blobs;
  std::mutex _generate_key_mutex;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlobStore);
};

} /* namespace inmemory */
} /* namespace blobstore */

#endif
