#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_

#include <cpp-utils/macros.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include "../cachingfsblobstore/CachingFsBlobStore.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class ParallelAccessFsBlobStoreAdapter final: public parallelaccessstore::ParallelAccessBaseStore<cachingfsblobstore::FsBlobRef, blockstore::Key> {
public:
  explicit ParallelAccessFsBlobStoreAdapter(cachingfsblobstore::CachingFsBlobStore *baseBlobStore)
    :_baseBlobStore(std::move(baseBlobStore)) {
  }

  boost::optional<cpputils::unique_ref<cachingfsblobstore::FsBlobRef>> loadFromBaseStore(const blockstore::Key &key) override {
	return _baseBlobStore->load(key);
  }

  void removeFromBaseStore(cpputils::unique_ref<cachingfsblobstore::FsBlobRef> block) override {
	return _baseBlobStore->remove(std::move(block));
  }

  void removeFromBaseStore(const blockstore::Key &key) override {
	return _baseBlobStore->remove(key);
  }

private:
  cachingfsblobstore::CachingFsBlobStore *_baseBlobStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStoreAdapter);
};

}
}

#endif
