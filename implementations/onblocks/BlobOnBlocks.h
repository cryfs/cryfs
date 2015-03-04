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
namespace datatreestore {
class DataTree;
}

class BlobOnBlocks: public Blob {
public:
  BlobOnBlocks(std::unique_ptr<datatreestore::DataTree> datatree);
  virtual ~BlobOnBlocks();

  blockstore::Key key() const override;

  uint64_t size() const override;
  void resize(uint64_t numBytes) override;

  void read(void *target, uint64_t offset, uint64_t size) const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

private:

  void traverseLeaves(uint64_t offsetBytes, uint64_t sizeBytes, std::function<void (uint64_t, void *, uint32_t)>) const;
  void resizeIfSmallerThan(uint64_t neededSize);

  std::unique_ptr<datatreestore::DataTree> _datatree;
};

}
}

#endif
