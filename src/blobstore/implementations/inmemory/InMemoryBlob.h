#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOB_H_
#define BLOBSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOB_H_

#include "blobstore/interface/Blob.h"
#include "blobstore/utils/Data.h"

namespace blobstore {
namespace inmemory {
class InMemoryBlobStore;

class InMemoryBlob: public Blob {
public:
  InMemoryBlob(size_t size);
  InMemoryBlob(const InMemoryBlob &rhs);
  virtual ~InMemoryBlob();

  void *data() override;
  const void *data() const override;

  void flush() override;

  size_t size() const override;

private:
  std::shared_ptr<Data> _data;
};

} /* namespace inmemory */
} /* namespace blobstore */

#endif
