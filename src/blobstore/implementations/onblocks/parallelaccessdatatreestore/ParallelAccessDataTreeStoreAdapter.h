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

class ParallelAccessDataTreeStoreAdapter final: public parallelaccessstore::ParallelAccessBaseStore<datatreestore::DataTree, blockstore::Key> {
public:
  ParallelAccessDataTreeStoreAdapter(datatreestore::DataTreeStore *baseDataTreeStore)
    :_baseDataTreeStore(std::move(baseDataTreeStore)) {
  }

  boost::optional<cpputils::unique_ref<datatreestore::DataTree>> loadFromBaseStore(const blockstore::Key &key) override {
	  return _baseDataTreeStore->load(key);
  }

  void removeFromBaseStore(cpputils::unique_ref<datatreestore::DataTree> dataTree) override {
	  return _baseDataTreeStore->remove(std::move(dataTree));
  }

private:
  datatreestore::DataTreeStore *_baseDataTreeStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessDataTreeStoreAdapter);
};

}
}
}

#endif
