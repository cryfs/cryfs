#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <cstring>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/DataUtils.h>

using std::shared_ptr;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;
using std::string;
using cpputils::Data;

namespace blockstore {
namespace testfake {

FakeBlock::FakeBlock(FakeBlockStore *store, const BlockId &blockId, shared_ptr<Data> data, bool dirty)
 : Block(blockId), _store(store), _data(data), _dataChanged(dirty) {
}

FakeBlock::~FakeBlock() {
  flush();
}

const void *FakeBlock::data() const {
  return _data->data();
}

void FakeBlock::write(const void *source, uint64_t offset, uint64_t size) {
  ASSERT(offset <= _data->size() && offset + size <= _data->size(), "Write outside of valid area"); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy(_data->dataOffset(offset), source, size);
  _dataChanged = true;
}

size_t FakeBlock::size() const {
  return _data->size();
}

void FakeBlock::resize(size_t newSize) {
  *_data = cpputils::DataUtils::resize(*_data, newSize);
  _dataChanged = true;
}

void FakeBlock::flush() {
  if(_dataChanged) {
    _store->updateData(blockId(), *_data);
    _dataChanged = false;
  }
}

}
}
