#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_

#include "../../interface/Blob.h"

#include <memory>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataLeafNode;
}
namespace cachingdatatreestore {
class CachedDataTreeRef;
}

class BlobOnBlocks: public Blob {
public:
  BlobOnBlocks(std::unique_ptr<cachingdatatreestore::CachedDataTreeRef> datatree);
  virtual ~BlobOnBlocks();

  blockstore::Key key() const override;

  uint64_t size() const override;
  void resize(uint64_t numBytes) override;

  void read(void *target, uint64_t offset, uint64_t size) const override;
  uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  std::unique_ptr<cachingdatatreestore::CachedDataTreeRef> releaseTree();

private:

  void traverseLeaves(uint64_t offsetBytes, uint64_t sizeBytes, std::function<void (uint64_t, datanodestore::DataLeafNode *, uint32_t, uint32_t)>) const;
  void resizeIfSmallerThan(uint64_t neededSize);

  std::unique_ptr<cachingdatatreestore::CachedDataTreeRef> _datatree;
};

}
}

#endif
