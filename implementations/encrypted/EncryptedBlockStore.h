#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <messmer/cpp-utils/macros.h>

namespace blockstore {
namespace encrypted {

class EncryptedBlockStore: public BlockStore {
public:
  EncryptedBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore);
};

}
}

#endif
