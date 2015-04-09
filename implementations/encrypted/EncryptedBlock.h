#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_

#include "../../interface/Block.h"
#include "EncryptionKey.h"
#include "../../utils/Data.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>

namespace blockstore {
namespace encrypted {
class EncryptedBlockStore;

class EncryptedBlock: public Block {
public:
  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  EncryptedBlock(std::unique_ptr<Block> baseBlock, const EncryptionKey &encKey);
  virtual ~EncryptedBlock();

  static std::unique_ptr<EncryptedBlock> CreateNew(std::unique_ptr<Block>, const EncryptionKey &encKey);

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;
  void flush() override;

  size_t size() const override;

  static constexpr unsigned int BASE_BLOCK_SIZE(unsigned int useableBlockSize) {
    return useableBlockSize + IV_SIZE;
  }

  static constexpr unsigned int USEABLE_BLOCK_SIZE(unsigned int baseBlockSize) {
    return baseBlockSize - IV_SIZE;
  }

private:
  std::unique_ptr<Block> _baseBlock;
  Data _plaintextData;
  EncryptionKey _encKey;
  bool _dataChanged;

  static constexpr unsigned int IV_SIZE = CryptoPP::AES::BLOCKSIZE;

  byte *baseBlockIV();
  byte *baseBlockData();

  void _encryptToBaseBlock();
  void _decryptFromBaseBlock();

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlock);
};

}
}

#endif
