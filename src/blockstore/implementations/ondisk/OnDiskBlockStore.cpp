#include <blockstore/implementations/ondisk/OnDiskBlock.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <blockstore/utils/RandomKeyGenerator.h>

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

unique_ptr<BlockWithKey> OnDiskBlockStore::create(const std::string &key, size_t size) {
  auto file_path = _rootdir / key;
  auto block = OnDiskBlock::CreateOnDisk(file_path, size);

  if (!block) {
    return nullptr;
  }
  return make_unique<BlockWithKey>(key, std::move(block));
}

unique_ptr<Block> OnDiskBlockStore::load(const string &key) {
  auto file_path = _rootdir / key;
  return OnDiskBlock::LoadFromDisk(file_path);
}

}
}
