#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_

#include "../../interface/BlobStore.h"
#include "messmer/blockstore/interface/BlockStore.h"

namespace blobstore {
namespace onblocks {
namespace parallelaccessdatatreestore {
class ParallelAccessDataTreeStore;
}

//TODO Make blobstore able to cope with incomplete data (some blocks missing, because they're not synchronized yet) and write test cases for that

class BlobStoreOnBlocks: public BlobStore {
public:
  BlobStoreOnBlocks(std::unique_ptr<blockstore::BlockStore> blockStore, uint32_t blocksizeBytes);
  virtual ~BlobStoreOnBlocks();

  std::unique_ptr<Blob> create() override;
  std::unique_ptr<Blob> load(const blockstore::Key &key) override;

  void remove(std::unique_ptr<Blob> blob) override;

private:
  std::unique_ptr<parallelaccessdatatreestore::ParallelAccessDataTreeStore> _dataTreeStore;
};

}
}

#endif
