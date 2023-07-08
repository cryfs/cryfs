#include "LowToHighLevelBlock.h"

using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
namespace DataUtils = cpputils::DataUtils;
using std::unique_lock;
using std::mutex;

namespace blockstore {
namespace lowtohighlevel {

optional<unique_ref<LowToHighLevelBlock>> LowToHighLevelBlock::TryCreateNew(BlockStore2 *baseBlockStore, const BlockId &blockId, Data data) {
  const bool success = baseBlockStore->tryCreate(blockId, data);
  if (!success) {
    return none;
  }

  return make_unique_ref<LowToHighLevelBlock>(blockId, std::move(data), baseBlockStore);
}

unique_ref<LowToHighLevelBlock> LowToHighLevelBlock::Overwrite(BlockStore2 *baseBlockStore, const BlockId &blockId, Data data) {
  baseBlockStore->store(blockId, data); // TODO Does it make sense to not store here, but only write back in the destructor of LowToHighLevelBlock? Also: What about tryCreate?
  return make_unique_ref<LowToHighLevelBlock>(blockId, std::move(data), baseBlockStore);
}

optional<unique_ref<LowToHighLevelBlock>> LowToHighLevelBlock::Load(BlockStore2 *baseBlockStore, const BlockId &blockId) {
  optional<Data> loadedData = baseBlockStore->load(blockId);
  if (loadedData == none) {
    return none;
  }
  return make_unique_ref<LowToHighLevelBlock>(blockId, std::move(*loadedData), baseBlockStore);
}

LowToHighLevelBlock::LowToHighLevelBlock(const BlockId &blockId, Data data, BlockStore2 *baseBlockStore)
    :Block(blockId),
     _baseBlockStore(baseBlockStore),
     _data(std::move(data)),
     _dataChanged(false),
     _mutex() {
}

LowToHighLevelBlock::~LowToHighLevelBlock() {
  const unique_lock<mutex> lock(_mutex);
  _storeToBaseBlock();
}

const void *LowToHighLevelBlock::data() const {
  return _data.data();
}

void LowToHighLevelBlock::write(const void *source, uint64_t offset, uint64_t count) {
  ASSERT(offset <= size() && offset + count <= size(), "Write outside of valid area"); //Also check offset < size() because of possible overflow in the addition
  std::memcpy(_data.dataOffset(offset), source, count);
  _dataChanged = true;
}

void LowToHighLevelBlock::flush() {
  const unique_lock<mutex> lock(_mutex);
  _storeToBaseBlock();
}

size_t LowToHighLevelBlock::size() const {
  return _data.size();
}

void LowToHighLevelBlock::resize(size_t newSize) {
  _data = DataUtils::resize(_data, newSize);
  _dataChanged = true;
}

void LowToHighLevelBlock::_storeToBaseBlock() {
  if (_dataChanged) {
    _baseBlockStore->store(blockId(), _data);
    _dataChanged = false;
  }
}


}
}
