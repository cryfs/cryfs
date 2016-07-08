#include "OnDiskBlock.h"
#include "OnDiskBlockStore.h"
#include <sys/statvfs.h>
#include <fspp/impl/Profiler.h>

using std::string;
using cpputils::Data;
using cpputils::unique_ref;
using boost::optional;
using boost::none;
using std::vector;

namespace bf = boost::filesystem;

namespace blockstore {
namespace ondisk {

    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile2(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile3(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile4(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile5(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile6(0);
    std::atomic<uint64_t> OnDiskBlockStore::loadFromDiskProfile7(0);

OnDiskBlockStore::OnDiskBlockStore(const boost::filesystem::path &rootdir)
 : _rootdir(rootdir), _numLoaded(0), _numCreated(0), _profile(0) {
  if (!bf::exists(rootdir)) {
    throw std::runtime_error("Base directory not found");
  }
  if (!bf::is_directory(rootdir)) {
      throw std::runtime_error("Base directory is not a directory");
  }
  //TODO Test for read access, write access, enter (x) access, and throw runtime_error in case
#ifndef CRYFS_NO_COMPATIBILITY
  _migrateBlockStore();
#endif
}

OnDiskBlockStore::~OnDiskBlockStore() {
  std::cout << "NumLoaded: " << _numLoaded << ", NumCreated: " << _numCreated << std::endl;
  std::cout << "Load from OnDisk: " << static_cast<double>(_profile)/1000000000 << std::endl;
  std::cout << "LoadFromDisk: " << static_cast<double>(loadFromDiskProfile)/1000000000 << std::endl;
  std::cout << "LoadFromDisk2: " << static_cast<double>(loadFromDiskProfile2)/1000000000 << std::endl;
  std::cout << "LoadFromDisk3: " << static_cast<double>(loadFromDiskProfile3)/1000000000 << std::endl;
  std::cout << "LoadFromDisk4: " << static_cast<double>(loadFromDiskProfile4)/1000000000 << std::endl;
  std::cout << "LoadFromDisk5: " << static_cast<double>(loadFromDiskProfile5)/1000000000 << std::endl;
  std::cout << "LoadFromDisk6: " << static_cast<double>(loadFromDiskProfile6)/1000000000 << std::endl;
  std::cout << "LoadFromDisk7: " << static_cast<double>(loadFromDiskProfile7)/1000000000 << std::endl;

}

#ifndef CRYFS_NO_COMPATIBILITY
void OnDiskBlockStore::_migrateBlockStore() {
  vector<string> blocksToMigrate;
  for (auto entry = bf::directory_iterator(_rootdir); entry != bf::directory_iterator(); ++entry) {
    if (bf::is_regular_file(entry->path()) && _isValidBlockKey(entry->path().filename().native())) {
      blocksToMigrate.push_back(entry->path().filename().native());
    }
  }
  if (blocksToMigrate.size() != 0) {
    std::cout << "Migrating CryFS filesystem..." << std::flush;
    for (auto key : blocksToMigrate) {
      Key::FromString(key); // Assert that it can be parsed as a key
      string dir = key.substr(0, 3);
      string file = key.substr(3);
      bf::create_directory(_rootdir / dir);
      bf::rename(_rootdir / key, _rootdir / dir / file);
    }
    std::cout << "done" << std::endl;
  }
}

bool OnDiskBlockStore::_isValidBlockKey(const string &key) {
  return key.size() == 32 && key.find_first_not_of("0123456789ABCDEF") == string::npos;
}
#endif

//TODO Do I have to lock tryCreate/remove and/or load? Or does ParallelAccessBlockStore take care of that?

optional<unique_ref<Block>> OnDiskBlockStore::tryCreate(const Key &key, Data data) {
  fspp::Profiler p(&_profile);
  ++_numCreated;
  //TODO Easier implementation? This is only so complicated because of the cast OnDiskBlock -> Block
  auto result = std::move(OnDiskBlock::CreateOnDisk(_rootdir, key, std::move(data)));
  if (result == boost::none) {
    return boost::none;
  }
  return unique_ref<Block>(std::move(*result));
}

optional<unique_ref<Block>> OnDiskBlockStore::load(const Key &key) {
  ++_numLoaded;
  return optional<unique_ref<Block>>(OnDiskBlock::LoadFromDisk(_rootdir, key));
}

void OnDiskBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  cpputils::destruct(std::move(block));
  OnDiskBlock::RemoveFromDisk(_rootdir, key);
}

uint64_t OnDiskBlockStore::numBlocks() const {
  uint64_t count = 0;
  for (auto prefixDir = bf::directory_iterator(_rootdir); prefixDir != bf::directory_iterator(); ++prefixDir) {
    if (bf::is_directory(prefixDir->path())) {
      count += std::distance(bf::directory_iterator(prefixDir->path()), bf::directory_iterator());
    }
  }
  return count;
}

uint64_t OnDiskBlockStore::estimateNumFreeBytes() const {
  struct statvfs stat;
  ::statvfs(_rootdir.c_str(), &stat);
  return stat.f_bsize*stat.f_bavail;
}

uint64_t OnDiskBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return OnDiskBlock::blockSizeFromPhysicalBlockSize(blockSize);
}

void OnDiskBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
  for (auto prefixDir = bf::directory_iterator(_rootdir); prefixDir != bf::directory_iterator(); ++prefixDir) {
    if (bf::is_directory(prefixDir->path())) {
      string blockKeyPrefix = prefixDir->path().filename().native();
      for (auto block = bf::directory_iterator(prefixDir->path()); block != bf::directory_iterator(); ++block) {
        string blockKeyPostfix = block->path().filename().native();
        callback(Key::FromString(blockKeyPrefix + blockKeyPostfix));
      }
    }
  }
}

}
}
