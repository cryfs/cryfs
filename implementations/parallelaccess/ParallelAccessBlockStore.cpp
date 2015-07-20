#include "BlockRef.h"
#include "ParallelAccessBlockStore.h"
#include "ParallelAccessBlockStoreAdapter.h"
#include <cassert>
#include <messmer/cpp-utils/pointer/cast.h>


using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::promise;
using cpputils::dynamic_pointer_move;
using cpputils::make_unique_ref;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {
namespace parallelaccess {

ParallelAccessBlockStore::ParallelAccessBlockStore(unique_ref<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)), _parallelAccessStore(make_unique_ref<ParallelAccessBlockStoreAdapter>(_baseBlockStore.get())) {
}

Key ParallelAccessBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

optional<unique_ref<Block>> ParallelAccessBlockStore::tryCreate(const Key &key, cpputils::Data data) {
  auto block = _baseBlockStore->tryCreate(key, std::move(data));
  if (block == none) {
	//TODO Test this code branch
	return none;
  }
  return unique_ref<Block>(_parallelAccessStore.add(key, std::move(*block)));
}

unique_ptr<Block> ParallelAccessBlockStore::load(const Key &key) {
  auto block = _parallelAccessStore.load(key);
  if (block == none) {
    return nullptr;
  }
  //TODO Don't use to_unique_ptr but make blockstore use unique_ref
  return cpputils::to_unique_ptr(std::move(*block));
}


void ParallelAccessBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  //TODO Don't use nullcheck but make blockstore use unique_ref
  auto block_ref = cpputils::nullcheck(std::move(block)).value();
  return _parallelAccessStore.remove(key, dynamic_pointer_move<BlockRef>(block_ref).value());
}

uint64_t ParallelAccessBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
