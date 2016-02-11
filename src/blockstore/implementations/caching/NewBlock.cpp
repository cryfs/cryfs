#include "NewBlock.h"
#include "CachingBlockStore.h"
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/DataUtils.h>

using cpputils::Data;
using boost::none;

namespace blockstore {
namespace caching {

NewBlock::NewBlock(const Key &key, Data data, CachingBlockStore *blockStore)
    :Block(key),
     _blockStore(blockStore),
     _data(std::move(data)),
     _baseBlock(none),
     _dataChanged(true) {
}

NewBlock::~NewBlock() {
  writeToBaseBlockIfChanged();
}

const void *NewBlock::data() const {
  return _data.data();
}

void NewBlock::write(const void *source, uint64_t offset, uint64_t size) {
  ASSERT(offset <= _data.size() && offset + size <= _data.size(), "Write outside of valid area");
  std::memcpy((uint8_t*)_data.data()+offset, source, size);
  _dataChanged = true;
}

void NewBlock::writeToBaseBlockIfChanged() {
  if (_dataChanged) {
    if (_baseBlock == none) {
      //TODO _data.copy() necessary?
      auto newBase = _blockStore->tryCreateInBaseStore(key(), _data.copy());
      ASSERT(newBase != boost::none, "Couldn't create base block"); //TODO What if tryCreate fails due to a duplicate key? We should ensure we don't use duplicate keys.
      _baseBlock = std::move(*newBase);
    } else {
        (*_baseBlock)->write(_data.data(), 0, _data.size());
    }
	_dataChanged = false;
  }
}

void NewBlock::remove() {
  if (_baseBlock != none) {
	_blockStore->removeFromBaseStore(std::move(*_baseBlock));
  }
  _dataChanged = false;
}

void NewBlock::flush() {
  writeToBaseBlockIfChanged();
  ASSERT(_baseBlock != none, "At this point, the base block should already have been created but wasn't");
  (*_baseBlock)->flush();
}

size_t NewBlock::size() const {
  return _data.size();
}

void NewBlock::resize(size_t newSize) {
    _data = cpputils::DataUtils::resize(std::move(_data), newSize);
    _dataChanged = true;
}

bool NewBlock::alreadyExistsInBaseStore() const {
  return _baseBlock != none;
}

}
}
