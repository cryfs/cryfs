#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_

#include "../../interface/BlobStore.h"
#include "BlobOnBlocks.h"
#include <blockstore/interface/BlockStore.h>

namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTreeStore;
}

//TODO Make blobstore able to cope with incomplete data (some blocks missing, because they're not synchronized yet) and write test cases for that

class BlobStoreOnBlocks final: public BlobStore {
public:
  BlobStoreOnBlocks(cpputils::unique_ref<blockstore::BlockStore> blockStore, uint64_t physicalBlocksizeBytes);
  ~BlobStoreOnBlocks();

  cpputils::unique_ref<Blob> create() override;
  boost::optional<cpputils::unique_ref<Blob>> load(const blockstore::BlockId &blockId) override;

  void remove(cpputils::unique_ref<Blob> blob) override;
  void remove(const blockstore::BlockId &blockId) override;

  //TODO Test blocksizeBytes/numBlocks/estimateSpaceForNumBlocksLeft
  //virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
  uint64_t virtualBlocksizeBytes() const override;
  uint64_t numBlocks() const override;
  uint64_t estimateSpaceForNumBlocksLeft() const override;

private:
  cpputils::unique_ref<datatreestore::DataTreeStore> _dataTreeStore;

  DISALLOW_COPY_AND_ASSIGN(BlobStoreOnBlocks);
};

}
}

#endif
