#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"

using std::unique_ptr;
using std::make_unique;
using std::string;
using cpputils::Data;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

OnDiskBlockStore::OnDiskBlockStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {}

unique_ptr<Block> OnDiskBlockStore::tryCreate(const Key &key, Data data) {
  auto block = OnDiskBlock::CreateOnDisk(_rootdir, key, std::move(data));

  if (!block) {
    return nullptr;
  }
  return std::move(block);
}

unique_ptr<Block> OnDiskBlockStore::load(const Key &key) {
  return OnDiskBlock::LoadFromDisk(_rootdir, key);
}

void OnDiskBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  block.reset();
  OnDiskBlock::RemoveFromDisk(_rootdir, key);
}

uint64_t OnDiskBlockStore::numBlocks() const {
  return std::distance(bf::directory_iterator(_rootdir), bf::directory_iterator());
}

}
}
