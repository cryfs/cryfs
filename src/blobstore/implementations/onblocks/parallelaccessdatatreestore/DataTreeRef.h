#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_

#include <parallelaccessstore/ParallelAccessStore.h>
#include "../datatreestore/DataTree.h"

namespace blobstore {
namespace onblocks {
namespace parallelaccessdatatreestore {

class DataTreeRef final: public parallelaccessstore::ParallelAccessStore<datatreestore::DataTree, DataTreeRef, blockstore::Key>::ResourceRefBase {
public:
  DataTreeRef(datatreestore::DataTree *baseTree): _baseTree(baseTree) {}

  const blockstore::Key &key() const {
    return _baseTree->key();
  }

  uint64_t maxBytesPerLeaf() const {
    return _baseTree->maxBytesPerLeaf();
  }

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func) {
    return _baseTree->traverseLeaves(beginIndex, endIndex, func);
  }

  uint32_t numLeaves() const {
    return _baseTree->numLeaves();
  }

  void resizeNumBytes(uint64_t newNumBytes) {
    return _baseTree->resizeNumBytes(newNumBytes);
  }

  uint64_t numStoredBytes() const {
    return _baseTree->numStoredBytes();
  }

  void flush() {
    return _baseTree->flush();
  }

private:

  datatreestore::DataTree *_baseTree;

  DISALLOW_COPY_AND_ASSIGN(DataTreeRef);
};

}
}
}

#endif
