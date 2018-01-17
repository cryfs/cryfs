#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_

#include "../../interface/Blob.h"
#include "datatreestore/LeafHandle.h"

#include <memory>
#include <boost/optional.hpp>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataLeafNode;
}
namespace datatreestore {
class DataTree;
}

class BlobOnBlocks final: public Blob {
public:
  BlobOnBlocks(cpputils::unique_ref<datatreestore::DataTree> datatree);
  ~BlobOnBlocks();

  const blockstore::BlockId &blockId() const override;

  uint64_t size() const override;
  void resize(uint64_t numBytes) override;

  cpputils::Data readAll() const override;
  void read(void *target, uint64_t offset, uint64_t size) const override;
  uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  cpputils::unique_ref<datatreestore::DataTree> releaseTree();

private:

  void _read(void *target, uint64_t offset, uint64_t count) const;
  void _traverseLeaves(uint64_t offsetBytes, uint64_t sizeBytes, std::function<void (uint64_t leafOffset, datatreestore::LeafHandle leaf, uint32_t begin, uint32_t count)> onExistingLeaf, std::function<cpputils::Data (uint64_t beginByte, uint32_t count)> onCreateLeaf) const;

  cpputils::unique_ref<datatreestore::DataTree> _datatree;
  mutable boost::optional<uint64_t> _sizeCache;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(BlobOnBlocks);
};

}
}

#endif
