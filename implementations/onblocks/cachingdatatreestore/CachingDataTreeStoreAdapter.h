#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_CACHINGDATATREESTOREADAPTER_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_CACHINGDATATREESTOREADAPTER_H_

#include <messmer/cpp-utils/macros.h>
#include <messmer/cachingstore/CachingStore.h>
#include "../datatreestore/DataTreeStore.h"
#include "../datatreestore/DataTree.h"

namespace blobstore {
namespace onblocks {
namespace cachingdatatreestore {

class CachingDataTreeStoreAdapter: public cachingstore::CachingBaseStore<datatreestore::DataTree, blockstore::Key> {
public:
  CachingDataTreeStoreAdapter(datatreestore::DataTreeStore *baseDataTreeStore)
    :_baseDataTreeStore(std::move(baseDataTreeStore)) {
  }

  std::unique_ptr<datatreestore::DataTree> loadFromBaseStore(const blockstore::Key &key) override {
	  return _baseDataTreeStore->load(key);
  }

  void removeFromBaseStore(std::unique_ptr<datatreestore::DataTree> dataTree) override {
	  return _baseDataTreeStore->remove(std::move(dataTree));
  }

private:
  datatreestore::DataTreeStore *_baseDataTreeStore;

  DISALLOW_COPY_AND_ASSIGN(CachingDataTreeStoreAdapter);
};

}
}
}

#endif
