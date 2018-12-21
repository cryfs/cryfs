#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_

#include <cpp-utils/macros.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include "cryfs/impl/filesystem/cachingfsblobstore/CachingFsBlobStore.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class ParallelAccessFsBlobStoreAdapter final: public parallelaccessstore::ParallelAccessBaseStore<cachingfsblobstore::FsBlobRef, blockstore::BlockId> {
public:
  explicit ParallelAccessFsBlobStoreAdapter(cachingfsblobstore::CachingFsBlobStore *baseBlobStore)
    :_baseBlobStore(baseBlobStore) {
  }

  boost::optional<cpputils::unique_ref<cachingfsblobstore::FsBlobRef>> loadFromBaseStore(const blockstore::BlockId &blockId) override {
	return _baseBlobStore->load(blockId);
  }

  void removeFromBaseStore(cpputils::unique_ref<cachingfsblobstore::FsBlobRef> block) override {
	return _baseBlobStore->remove(std::move(block));
  }

  void removeFromBaseStore(const blockstore::BlockId &blockId) override {
	return _baseBlobStore->remove(blockId);
  }

private:
  cachingfsblobstore::CachingFsBlobStore *_baseBlobStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStoreAdapter);
};

}
}

#endif
