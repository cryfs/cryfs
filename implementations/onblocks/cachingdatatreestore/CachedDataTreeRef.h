#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_DATATREEREF_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_DATATREEREF_H_

#include "../datatreestore/DataTree.h"
#include <messmer/cachingstore/CachingStore.h>

namespace blobstore {
namespace onblocks {
namespace cachingdatatreestore {

class CachedDataTreeRef: public cachingstore::CachingStore<datatreestore::DataTree, CachedDataTreeRef, blockstore::Key>::CachedResource {
public:
  CachedDataTreeRef(datatreestore::DataTree *baseTree): _baseTree(baseTree) {}

  const blockstore::Key &key() const {
    return _baseTree->key();
  }

  uint32_t maxBytesPerLeaf() const {
    return _baseTree->maxBytesPerLeaf();
  }

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (const datanodestore::DataLeafNode*, uint32_t)> func) const {
    return const_cast<const datatreestore::DataTree*>(_baseTree)->traverseLeaves(beginIndex, endIndex, func);
  }

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func) {
    return _baseTree->traverseLeaves(beginIndex, endIndex, func);
  }

  void resizeNumBytes(uint64_t newNumBytes) {
    return _baseTree->resizeNumBytes(newNumBytes);
  }

  uint64_t numStoredBytes() const {
    return _baseTree->numStoredBytes();
  }
private:
  datatreestore::DataTree *_baseTree;
};

}
}
}

#endif
