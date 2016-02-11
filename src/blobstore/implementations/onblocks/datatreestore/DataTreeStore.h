#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_

#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blockstore/utils/Key.h>
#include <boost/optional.hpp>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
}
namespace datatreestore {
class DataTree;

class DataTreeStore final {
public:
  DataTreeStore(cpputils::unique_ref<datanodestore::DataNodeStore> nodeStore);
  ~DataTreeStore();

  boost::optional<cpputils::unique_ref<DataTree>> load(const blockstore::Key &key);

  cpputils::unique_ref<DataTree> createNewTree();

  void remove(cpputils::unique_ref<DataTree> tree);

private:
  cpputils::unique_ref<datanodestore::DataNodeStore> _nodeStore;

  DISALLOW_COPY_AND_ASSIGN(DataTreeStore);
};

}
}
}

#endif
