#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <cstring>

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

FakeBlock::FakeBlock(FakeBlockStore *store, const Key &key, shared_ptr<Data> data, bool dirty)
 : Block(key), _store(store), _data(data), _dataChanged(dirty) {
}

FakeBlock::~FakeBlock() {
  flush();
}

const void *FakeBlock::data() const {
  return _data->data();
}

void FakeBlock::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= _data->size() && offset + size <= _data->size()); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data->data()+offset, source, size);
  _dataChanged = true;
}

size_t FakeBlock::size() const {
  return _data->size();
}

void FakeBlock::flush() {
  if(_dataChanged) {
    _store->updateData(key(), *_data);
    _dataChanged = false;
  }
}

}
}
