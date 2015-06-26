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
  BlobStoreOnBlocks(cpputils::unique_ref<blockstore::BlockStore> blockStore, uint32_t blocksizeBytes);
  virtual ~BlobStoreOnBlocks();

  cpputils::unique_ref<Blob> create() override;
  boost::optional<cpputils::unique_ref<Blob>> load(const blockstore::Key &key) override;

  void remove(cpputils::unique_ref<Blob> blob) override;

private:
  cpputils::unique_ref<parallelaccessdatatreestore::ParallelAccessDataTreeStore> _dataTreeStore;
};

}
}

#endif
