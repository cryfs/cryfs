#include <blockstore/implementations/ondisk/OnDiskBlock.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

OnDiskBlockStore::OnDiskBlockStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir) {}

unique_ptr<Block> OnDiskBlockStore::create(const Key &key, size_t size) {
  auto block = OnDiskBlock::CreateOnDisk(_rootdir, key, size);

  if (!block) {
    return nullptr;
  }
  return std::move(block);
}

unique_ptr<Block> OnDiskBlockStore::load(const Key &key) {
  return OnDiskBlock::LoadFromDisk(_rootdir, key);
}

}
}
