#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_

#include <parallelaccessstore/ParallelAccessStore.h>
#include "../datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datatreestore/LeafHandle.h"

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

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (uint32_t index, datatreestore::LeafHandle leaf)> onExistingLeaf, std::function<cpputils::Data (uint32_t index)> onCreateLeaf) {
    return _baseTree->traverseLeaves(beginIndex, endIndex, onExistingLeaf, onCreateLeaf);
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
