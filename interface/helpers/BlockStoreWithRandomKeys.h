#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTOREWITHRANDOMKEYS_H_
#define FSPP_BLOCKSTORE_BLOCKSTOREWITHRANDOMKEYS_H_

#include "../BlockStore.h"
#include "../Block.h"

namespace blockstore {

// This is an implementation helpers for BlockStores that use random block keys.
// You should never give this static type to the client. The client should always
// work with the BlockStore interface instead.
class BlockStoreWithRandomKeys: public BlockStore {
public:
  //TODO Use boost::optional (if key already exists)
  // Return nullptr if key already exists
  virtual std::unique_ptr<Block> create(const Key &key, size_t size) = 0;

  std::unique_ptr<Block> create(size_t size) final;

private:
  std::unique_ptr<Block> tryCreate(size_t size);
};

}

#endif
