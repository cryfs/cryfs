#include "EncryptedBlock.h"
#include <cryptopp/cryptopp/modes.h>

#include "../../utils/BlockStoreUtils.h"

using CryptoPP::CFB_Mode;
using CryptoPP::AES;

using std::make_unique;

//TODO not only encryption, but also hmac

namespace blockstore {
namespace encrypted {

constexpr unsigned int EncryptedBlock::IV_SIZE;

std::unique_ptr<EncryptedBlock> EncryptedBlock::TryCreateNew(BlockStore *baseBlockStore, const Key &key, Data data, const EncryptionKey &encKey) {
  Data encrypted = _encrypt(data, encKey);
  auto baseBlock = baseBlockStore->tryCreate(key, std::move(encrypted));
  if (baseBlock.get() == nullptr) {
	//TODO Test this code branch
	return nullptr;
  }

  return make_unique<EncryptedBlock>(std::move(baseBlock), encKey);
}

EncryptedBlock::EncryptedBlock(std::unique_ptr<Block> baseBlock, const EncryptionKey &encKey)
    :Block(baseBlock->key()),
     _baseBlock(std::move(baseBlock)),
     _plaintextData(USEABLE_BLOCK_SIZE(_baseBlock->size())),
     _encKey(encKey),
     _dataChanged(false) {
  _decryptFromBaseBlock();
}

EncryptedBlock::~EncryptedBlock() {
  _encryptToBaseBlock();
}

const void *EncryptedBlock::data() const {
  return _plaintextData.data();
}

void EncryptedBlock::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= _plaintextData.size() && offset + size <= _plaintextData.size()); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_plaintextData.data()+offset, source, size);
  _dataChanged = true;
}

void EncryptedBlock::flush() {
  _encryptToBaseBlock();
  return _baseBlock->flush();
}

size_t EncryptedBlock::size() const {
  return _plaintextData.size();
}

void EncryptedBlock::_decryptFromBaseBlock() {
  const byte *iv = (byte*)_baseBlock->data();
  const byte *data = (byte*)_baseBlock->data() + IV_SIZE;
  auto decryption = CFB_Mode<AES>::Decryption((byte*)_encKey.data(), EncryptionKey::BINARY_LENGTH, iv);
  decryption.ProcessData((byte*)_plaintextData.data(), data, _plaintextData.size());
}

void EncryptedBlock::_encryptToBaseBlock() {
  if (_dataChanged) {
	Data encrypted = _encrypt(_plaintextData, _encKey);
    _baseBlock->write(encrypted.data(), 0, encrypted.size());
    _dataChanged = false;
  }
}

Data EncryptedBlock::_encrypt(const Data &plaintext, const EncryptionKey &encKey) {
  FixedSizeData<IV_SIZE> iv = FixedSizeData<IV_SIZE>::CreateRandom();
  auto encryption = CFB_Mode<AES>::Encryption(encKey.data(), EncryptionKey::BINARY_LENGTH, iv.data());
  //TODO More performance when not using "Data encrypted" object, but encrypting directly to a target that was specified via a parameter using a specialized CryptoPP sink
  Data encrypted(IV_SIZE + plaintext.size());
  std::memcpy(encrypted.data(), iv.data(), IV_SIZE);
  encryption.ProcessData((byte*)encrypted.data() + IV_SIZE, (byte*)plaintext.data(), plaintext.size());
  return encrypted;
}

}
}
