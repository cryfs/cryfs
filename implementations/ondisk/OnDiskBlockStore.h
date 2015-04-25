#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_

#include <boost/filesystem.hpp>
#include "../../interface/helpers/BlockStoreWithRandomKeys.h"

#include "messmer/cpp-utils/macros.h"

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore: public BlockStoreWithRandomKeys {
public:
  OnDiskBlockStore(const boost::filesystem::path &rootdir);

  std::unique_ptr<Block> tryCreate(const Key &key, cpputils::Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  const boost::filesystem::path _rootdir;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore);
};

}
}

#endif
