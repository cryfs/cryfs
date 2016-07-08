#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_

#include <boost/filesystem.hpp>
#include "../../interface/helpers/BlockStoreWithRandomKeys.h"

#include <cpp-utils/macros.h>
#include <atomic>
#include <cpp-utils/data/Data.h>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore final: public BlockStoreWithRandomKeys {
public:
  explicit OnDiskBlockStore(const boost::filesystem::path &rootdir);
  ~OnDiskBlockStore();

  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  //TODO Can we make this faster by allowing to delete blocks by only having theiy Key? So we wouldn't have to load it first?
  void remove(cpputils::unique_ref<Block> block) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

    static std::atomic<uint64_t> loadFromDiskProfile;
    static std::atomic<uint64_t> loadFromDiskProfile2;
    static std::atomic<uint64_t> loadFromDiskProfile3;
    static std::atomic<uint64_t> loadFromDiskProfile4;
    static std::atomic<uint64_t> loadFromDiskProfile5;
    static std::atomic<uint64_t> loadFromDiskProfile6;
    static std::atomic<uint64_t> loadFromDiskProfile7;

private:
  const boost::filesystem::path _rootdir;
#ifndef CRYFS_NO_COMPATIBILITY
  void _migrateBlockStore();
  bool _isValidBlockKey(const std::string &key);
#endif
  std::atomic<uint32_t> _numLoaded;
  std::atomic<uint32_t> _numCreated;
        mutable std::atomic<uint64_t> _profile;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore);
};

}
}

#endif
