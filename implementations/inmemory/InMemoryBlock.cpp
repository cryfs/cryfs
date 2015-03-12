#include "InMemoryBlock.h"
#include "InMemoryBlockStore.h"
#include <cstring>

using std::unique_ptr;
using std::make_shared;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;

namespace blockstore {
namespace inmemory {

InMemoryBlock::InMemoryBlock(const Key &key, size_t size)
 : Block(key), _data(make_shared<Data>(size)) {
  _data->FillWithZeroes();
}

InMemoryBlock::InMemoryBlock(const InMemoryBlock &rhs)
 : Block(rhs), _data(rhs._data) {
}

InMemoryBlock::~InMemoryBlock() {
}

const void *InMemoryBlock::data() const {
  return _data->data();
}

void InMemoryBlock::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= _data->size() && offset + size <= _data->size()); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data->data()+offset, source, size);
}

size_t InMemoryBlock::size() const {
  return _data->size();
}

void InMemoryBlock::flush() {
}

}
}
