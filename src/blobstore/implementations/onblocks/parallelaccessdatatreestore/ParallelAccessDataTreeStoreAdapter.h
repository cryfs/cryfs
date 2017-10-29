#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTOREADAPTER_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTOREADAPTER_H_

#include <cpp-utils/macros.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include "../datatreestore/DataTreeStore.h"
#include "../datatreestore/DataTree.h"

namespace blobstore {
namespace onblocks {
namespace parallelaccessdatatreestore {

class ParallelAccessDataTreeStoreAdapter final: public parallelaccessstore::ParallelAccessBaseStore<datatreestore::DataTree, blockstore::BlockId> {
public:
  ParallelAccessDataTreeStoreAdapter(datatreestore::DataTreeStore *baseDataTreeStore)
    :_baseDataTreeStore(baseDataTreeStore) {
  }

  boost::optional<cpputils::unique_ref<datatreestore::DataTree>> loadFromBaseStore(const blockstore::BlockId &blockId) override {
	  return _baseDataTreeStore->load(blockId);
  }

  void removeFromBaseStore(cpputils::unique_ref<datatreestore::DataTree> dataTree) override {
	  return _baseDataTreeStore->remove(std::move(dataTree));
  }

  void removeFromBaseStore(const blockstore::BlockId &blockId) override {
    return _baseDataTreeStore->remove(blockId);
  }

private:
  datatreestore::DataTreeStore *_baseDataTreeStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessDataTreeStoreAdapter);
};

}
}
}

#endif
