#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include <blockstore/utils/Key.h>
#include <parallelaccessstore/ParallelAccessStore.h>

namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTreeStore;
class DataTree;
}
namespace parallelaccessdatatreestore {
class DataTreeRef;

//TODO Test CachingDataTreeStore

class ParallelAccessDataTreeStore final {
public:
  ParallelAccessDataTreeStore(cpputils::unique_ref<datatreestore::DataTreeStore> dataTreeStore);
  ~ParallelAccessDataTreeStore();

  boost::optional<cpputils::unique_ref<DataTreeRef>> load(const blockstore::Key &key);

  cpputils::unique_ref<DataTreeRef> createNewTree();

  void remove(cpputils::unique_ref<DataTreeRef> tree);

private:
  cpputils::unique_ref<datatreestore::DataTreeStore> _dataTreeStore;
  parallelaccessstore::ParallelAccessStore<datatreestore::DataTree, DataTreeRef, blockstore::Key> _parallelAccessStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessDataTreeStore);
};

}
}
}

#endif
