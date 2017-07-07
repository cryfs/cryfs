#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"
#include "../../interface/BlockStore2.h"

#include <cpp-utils/macros.h>
#include <memory>
#include <iostream>
#include <boost/optional.hpp>
#include <cpp-utils/crypto/symmetric/Cipher.h>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/DataUtils.h>
#include <mutex>
#include <cpp-utils/logging/logging.h>
#include "LowToHighLevelBlockStore.h"

namespace blockstore {
namespace lowtohighlevel {

class LowToHighLevelBlock final: public Block {
public:
  static boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> TryCreateNew(BlockStore2 *baseBlockStore, const Key &key, cpputils::Data data);
  static cpputils::unique_ref<LowToHighLevelBlock> Overwrite(BlockStore2 *baseBlockStore, const Key &key, cpputils::Data data);
  static boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> Load(BlockStore2 *baseBlockStore, const Key &key);

  LowToHighLevelBlock(const Key& key, cpputils::Data data, BlockStore2 *baseBlockStore);
  ~LowToHighLevelBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

private:
  BlockStore2 *_baseBlockStore;
  cpputils::Data _data;
  bool _dataChanged;
  std::mutex _mutex;

  void _storeToBaseBlock();

  DISALLOW_COPY_AND_ASSIGN(LowToHighLevelBlock);
};


inline boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> LowToHighLevelBlock::TryCreateNew(BlockStore2 *baseBlockStore, const Key &key, cpputils::Data data) {
  // TODO .get() is blocking
  bool success = baseBlockStore->tryCreate(key, data.copy()).get(); // TODO Copy necessary?
  if (!success) {
    return boost::none;
  }

  return cpputils::make_unique_ref<LowToHighLevelBlock>(key, std::move(data), baseBlockStore);
}

inline cpputils::unique_ref<LowToHighLevelBlock> LowToHighLevelBlock::Overwrite(BlockStore2 *baseBlockStore, const Key &key, cpputils::Data data) {
  auto baseBlock = baseBlockStore->store(key, data); // TODO Does it make sense to not store here, but only write back in the destructor of LowToHighLevelBlock? Also: What about tryCreate?
  return cpputils::make_unique_ref<LowToHighLevelBlock>(key, std::move(data), baseBlockStore);
}

inline boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> LowToHighLevelBlock::Load(BlockStore2 *baseBlockStore, const Key &key) {
  boost::optional<cpputils::Data> loadedData = baseBlockStore->load(key).get(); // TODO .get() is blocking
  if (loadedData == boost::none) {
    return boost::none;
  }
  return cpputils::make_unique_ref<LowToHighLevelBlock>(key, std::move(*loadedData), baseBlockStore);
}

inline LowToHighLevelBlock::LowToHighLevelBlock(const Key& key, cpputils::Data data, BlockStore2 *baseBlockStore)
    :Block(key),
   _baseBlockStore(baseBlockStore),
   _data(std::move(data)),
   _dataChanged(false),
   _mutex() {
}

inline LowToHighLevelBlock::~LowToHighLevelBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _storeToBaseBlock();
}

inline const void *LowToHighLevelBlock::data() const {
  return (uint8_t*)_data.data();
}

inline void LowToHighLevelBlock::write(const void *source, uint64_t offset, uint64_t count) {
  ASSERT(offset <= size() && offset + count <= size(), "Write outside of valid area"); //Also check offset < size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data.data()+offset, source, count);
  _dataChanged = true;
}

inline void LowToHighLevelBlock::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  _storeToBaseBlock();
}

inline size_t LowToHighLevelBlock::size() const {
  return _data.size();
}

inline void LowToHighLevelBlock::resize(size_t newSize) {
  _data = cpputils::DataUtils::resize(std::move(_data), newSize);
  _dataChanged = true;
}

inline void LowToHighLevelBlock::_storeToBaseBlock() {
  if (_dataChanged) {
    _baseBlockStore->store(key(), _data);
    _dataChanged = false;
  }
}

}
}

#endif
