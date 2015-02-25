#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_

#include <messmer/cpp-utils/macros.h>

namespace blobstore{
namespace onblocks{
namespace datanodestore{

struct DataInnerNode_ChildEntry {
public:
  blockstore::Key key() const {
    return blockstore::Key::FromBinary(_keydata);
  }
private:
  void setKey(const blockstore::Key &key) {
    key.ToBinary(_keydata);
  }
  friend class DataInnerNode;
  uint8_t _keydata[blockstore::Key::KEYLENGTH_BINARY];
  DISALLOW_COPY_AND_ASSIGN(DataInnerNode_ChildEntry);
};

}
}
}

#endif
