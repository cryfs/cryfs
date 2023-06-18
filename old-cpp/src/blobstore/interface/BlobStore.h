#pragma once
#ifndef MESSMER_BLOBSTORE_INTERFACE_BLOBSTORE_H_
#define MESSMER_BLOBSTORE_INTERFACE_BLOBSTORE_H_

#include "Blob.h"
#include <string>
#include <memory>

#include <blockstore/utils/BlockId.h>
#include <cpp-utils/pointer/unique_ref.h>

namespace blobstore {

//TODO Remove this interface. We'll only use BlobStoreOnBlocks and never a different one. Rename BlobStoreOnBlocks to simply BlobStore.
class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual cpputils::unique_ref<Blob> create() = 0;
  virtual boost::optional<cpputils::unique_ref<Blob>> load(const blockstore::BlockId &blockId) = 0;
  virtual void remove(cpputils::unique_ref<Blob> blob) = 0;
  virtual void remove(const blockstore::BlockId &blockId) = 0;

  virtual uint64_t numBlocks() const = 0;
  virtual uint64_t estimateSpaceForNumBlocksLeft() const = 0;
  //virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
  virtual uint64_t virtualBlocksizeBytes() const = 0;
};

}

#endif
