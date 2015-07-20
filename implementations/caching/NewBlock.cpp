#include "NewBlock.h"
#include "CachingBlockStore.h"

using std::unique_ptr;
using cpputils::Data;

namespace blockstore {
namespace caching {

NewBlock::NewBlock(const Key &key, Data data, CachingBlockStore *blockStore)
    :Block(key),
     _blockStore(blockStore),
     _data(std::move(data)),
     _baseBlock(nullptr),
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
    if (_baseBlock.get() == nullptr) {
      //TODO _data.copy() necessary?
      auto newBase = _blockStore->tryCreateInBaseStore(key(), _data.copy());
      assert(newBase != boost::none); //TODO What if tryCreate fails due to a duplicate key? We should ensure we don't use duplicate keys.
      //TODO Don't use to_unique_ptr but make _baseBlock a unique_ref
      _baseBlock = cpputils::to_unique_ptr(std::move(*newBase));
    } else {
	  _baseBlock->write(_data.data(), 0, _data.size());
    }
	_dataChanged = false;
  }
}

void NewBlock::remove() {
  if (_baseBlock.get() != nullptr) {
	_blockStore->removeFromBaseStore(std::move(_baseBlock));
  }
  _dataChanged = false;
}

void NewBlock::flush() {
  writeToBaseBlockIfChanged();
  _baseBlock->flush();
}

size_t NewBlock::size() const {
  return _data.size();
}

bool NewBlock::alreadyExistsInBaseStore() const {
  return _baseBlock.get() != nullptr;
}

}
}
