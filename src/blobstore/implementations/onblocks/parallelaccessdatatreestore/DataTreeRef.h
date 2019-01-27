#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_DATATREEREF_H_

#include <parallelaccessstore/ParallelAccessStore.h>
#include "../datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datatreestore/LeafHandle.h"

namespace blobstore {
namespace onblocks {
namespace parallelaccessdatatreestore {

class DataTreeRef final: public parallelaccessstore::ParallelAccessStore<datatreestore::DataTree, DataTreeRef, blockstore::BlockId>::ResourceRefBase {
public:
  DataTreeRef(datatreestore::DataTree *baseTree): _baseTree(baseTree) {}

  const blockstore::BlockId &blockId() const {
    return _baseTree->blockId();
  }

  uint64_t maxBytesPerLeaf() const {
    return _baseTree->maxBytesPerLeaf();
  }

  uint32_t numLeaves() const {
    return _baseTree->numLeaves();
  }

  void resizeNumBytes(uint64_t newNumBytes) {
    return _baseTree->resizeNumBytes(newNumBytes);
  }

  uint64_t numBytes() const {
    return _baseTree->numBytes();
  }

  uint64_t tryReadBytes(void *target, uint64_t offset, uint64_t count) const {
    return _baseTree->tryReadBytes(target, offset, count);
  }

  void readBytes(void *target, uint64_t offset, uint64_t count) const {
    return _baseTree->readBytes(target, offset, count);
  }

  cpputils::Data readAllBytes() const {
    return _baseTree->readAllBytes();
  }

  void writeBytes(const void *source, uint64_t offset, uint64_t count) {
    return _baseTree->writeBytes(source, offset, count);
  }

  void flush() {
    return _baseTree->flush();
  }

  uint32_t numNodes() const {
    return _baseTree->numNodes();
  }

private:

  datatreestore::DataTree *_baseTree;

  DISALLOW_COPY_AND_ASSIGN(DataTreeRef);
};

}
}
}

#endif
