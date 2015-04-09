#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_

#include "../../interface/Block.h"
#include "EncryptionKey.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>

namespace blockstore {
namespace encrypted {
class EncryptedBlockStore;

class EncryptedBlock: public Block {
public:
  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  EncryptedBlock(std::unique_ptr<Block> baseBlock, const EncryptionKey &encKey);

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;
  void flush() override;

  size_t size() const override;

private:
  std::unique_ptr<Block> _baseBlock;
  EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlock);
};

}
}

#endif
