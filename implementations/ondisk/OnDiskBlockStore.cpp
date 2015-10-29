#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"

using std::string;
using cpputils::Data;
using cpputils::unique_ref;
using boost::optional;
using boost::none;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

OnDiskBlockStore::OnDiskBlockStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {
  if (!bf::exists(rootdir)) {
    throw std::runtime_error("Base directory not found");
  }
  //TODO Test for read access, write access, enter (x) access, and throw runtime_error in case
}

//TODO Do I have to lock tryCreate/remove and/or load? Or does ParallelAccessBlockStore take care of that?

optional<unique_ref<Block>> OnDiskBlockStore::tryCreate(const Key &key, Data data) {
  //TODO Easier implementation? This is only so complicated because of the cast OnDiskBlock -> Block
  auto result = std::move(OnDiskBlock::CreateOnDisk(_rootdir, key, std::move(data)));
  if (result == boost::none) {
    return boost::none;
  }
  return unique_ref<Block>(std::move(*result));
}

optional<unique_ref<Block>> OnDiskBlockStore::load(const Key &key) {
  return optional<unique_ref<Block>>(OnDiskBlock::LoadFromDisk(_rootdir, key));
}

void OnDiskBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  cpputils::destruct(std::move(block));
  OnDiskBlock::RemoveFromDisk(_rootdir, key);
}

uint64_t OnDiskBlockStore::numBlocks() const {
  return std::distance(bf::directory_iterator(_rootdir), bf::directory_iterator());
}

}
}
