#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <cstring>

using std::unique_ptr;
using std::shared_ptr;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;
using std::string;

namespace blockstore {
namespace testfake {

FakeBlock::FakeBlock(FakeBlockStore *store, const string &key, shared_ptr<Data> data)
 : _store(store), _key(key), _data(data) {
}

FakeBlock::~FakeBlock() {
  flush();
}

void *FakeBlock::data() {
  return _data->data();
}

const void *FakeBlock::data() const {
  return _data->data();
}

size_t FakeBlock::size() const {
  return _data->size();
}

void FakeBlock::flush() {
  _store->updateData(_key, *_data);
}

}
}
