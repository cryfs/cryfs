#include <blockstore/implementations/inmemory/InMemoryBlock.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <cstring>

using std::unique_ptr;
using std::make_unique;
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

void *InMemoryBlock::data() {
  return _data->data();
}

const void *InMemoryBlock::data() const {
  return _data->data();
}

size_t InMemoryBlock::size() const {
  return _data->size();
}

void InMemoryBlock::flush() {
}

}
}
