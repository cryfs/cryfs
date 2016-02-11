#include "InMemoryBlock.h"
#include "InMemoryBlockStore.h"
#include <cstring>
#include <cpp-utils/data/DataUtils.h>
#include <cpp-utils/assert/assert.h>

using std::make_shared;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;
using cpputils::Data;

namespace blockstore {
namespace inmemory {

InMemoryBlock::InMemoryBlock(const Key &key, Data data)
 : Block(key), _data(make_shared<Data>(std::move(data))) {
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
  ASSERT(offset <= _data->size() && offset + size <= _data->size(), "Write outside of valid area"); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_data->data()+offset, source, size);
}

size_t InMemoryBlock::size() const {
  return _data->size();
}

void InMemoryBlock::resize(size_t newSize) {
    *_data = cpputils::DataUtils::resize(std::move(*_data), newSize);
}

void InMemoryBlock::flush() {
}

}
}
