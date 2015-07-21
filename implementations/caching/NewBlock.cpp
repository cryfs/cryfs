#include "NewBlock.h"
#include "CachingBlockStore.h"

using std::unique_ptr;
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
  assert(offset <= _data.size() && offset + size <= _data.size());
  std::memcpy((uint8_t*)_data.data()+offset, source, size);
  _dataChanged = true;
}

void NewBlock::writeToBaseBlockIfChanged() {
  if (_dataChanged) {
    if (_baseBlock == none) {
      //TODO _data.copy() necessary?
      auto newBase = _blockStore->tryCreateInBaseStore(key(), _data.copy());
      assert(newBase != boost::none); //TODO What if tryCreate fails due to a duplicate key? We should ensure we don't use duplicate keys.
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
  assert(_baseBlock != none);
  (*_baseBlock)->flush();
}

size_t NewBlock::size() const {
  return _data.size();
}

bool NewBlock::alreadyExistsInBaseStore() const {
  return _baseBlock != none;
}

}
}
