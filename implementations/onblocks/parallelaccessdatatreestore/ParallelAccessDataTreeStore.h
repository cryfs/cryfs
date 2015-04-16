#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_

#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <messmer/blockstore/utils/Key.h>
#include <messmer/parallelaccessstore/ParallelAccessStore.h>

namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTreeStore;
class DataTree;
}
namespace parallelaccessdatatreestore {
class DataTreeRef;

//TODO Test CachingDataTreeStore

class ParallelAccessDataTreeStore {
public:
  ParallelAccessDataTreeStore(std::unique_ptr<datatreestore::DataTreeStore> dataTreeStore);
  virtual ~ParallelAccessDataTreeStore();

  std::unique_ptr<DataTreeRef> load(const blockstore::Key &key);

  std::unique_ptr<DataTreeRef> createNewTree();

  void remove(std::unique_ptr<DataTreeRef> tree);

private:
  std::unique_ptr<datatreestore::DataTreeStore> _dataTreeStore;
  parallelaccessstore::ParallelAccessStore<datatreestore::DataTree, DataTreeRef, blockstore::Key> _parallelAccessStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessDataTreeStore);
};

}
}
}

#endif
