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
  explicit ParallelAccessFsBlobStoreAdapter(cachingfsblobstore::CachingFsBlobStore *baseBlockStore)
    :_baseBlockStore(std::move(baseBlockStore)) {
  }

  boost::optional<cpputils::unique_ref<cachingfsblobstore::FsBlobRef>> loadFromBaseStore(const blockstore::Key &key) override {
	return _baseBlockStore->load(key);
  }

  void removeFromBaseStore(cpputils::unique_ref<cachingfsblobstore::FsBlobRef> block) override {
	return _baseBlockStore->remove(std::move(block));
  }

private:
  cachingfsblobstore::CachingFsBlobStore *_baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStoreAdapter);
};

}
}

#endif
