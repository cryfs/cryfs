#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE_H_

#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>
#include <boost/filesystem.hpp>

#include "fspp/utils/macros.h"

#include <mutex>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore: public BlockStoreWithRandomKeys {
public:
  OnDiskBlockStore(const boost::filesystem::path &rootdir);

  std::unique_ptr<BlockWithKey> create(const std::string &key, size_t size) override;
  std::unique_ptr<Block> load(const std::string &key) override;

private:
  const boost::filesystem::path _rootdir;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore);
};

}
}

#endif
