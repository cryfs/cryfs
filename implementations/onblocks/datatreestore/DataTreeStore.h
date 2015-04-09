#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_

#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <messmer/blockstore/utils/Key.h>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
}
namespace datatreestore {
class DataTree;

class DataTreeStore {
public:
  DataTreeStore(std::unique_ptr<datanodestore::DataNodeStore> nodeStore);
  virtual ~DataTreeStore();

  std::unique_ptr<DataTree> load(const blockstore::Key &key);

  std::unique_ptr<DataTree> createNewTree();

  void remove(std::unique_ptr<DataTree> tree);

private:
  std::unique_ptr<datanodestore::DataNodeStore> _nodeStore;

  DISALLOW_COPY_AND_ASSIGN(DataTreeStore);
};

}
}
}

#endif
