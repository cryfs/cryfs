#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_

#include <boost/filesystem.hpp>
#include "../../interface/helpers/BlockStoreWithRandomKeys.h"

#include <cpp-utils/macros.h>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore final: public BlockStoreWithRandomKeys {
public:
  explicit OnDiskBlockStore(const boost::filesystem::path &rootdir);

  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  //TODO Can we make this faster by allowing to delete blocks by only having theiy Key? So we wouldn't have to load it first?
  void remove(cpputils::unique_ref<Block> block) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;

private:
  const boost::filesystem::path _rootdir;
#ifndef CRYFS_NO_COMPATIBILITY
  void _migrateBlockStore();
  bool _isValidBlockKey(const std::string &key);
#endif

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore);
};

}
}

#endif
