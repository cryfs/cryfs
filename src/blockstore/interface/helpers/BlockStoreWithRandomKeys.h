#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTOREWITHRANDOMKEYS_H_
#define FSPP_BLOCKSTORE_BLOCKSTOREWITHRANDOMKEYS_H_

#include <blockstore/interface/Block.h>
#include <blockstore/interface/BlockStore.h>

namespace blockstore {

// This is an implementation helpers for BlockStores that use random block keys.
// You should never give this static type to the client. The client should always
// work with the BlockStore interface instead.
class BlockStoreWithRandomKeys: public BlockStore {
public:
  //TODO Use boost::optional (if key already exists)
  // Return nullptr if key already exists
  virtual std::unique_ptr<Block> create(const Key &key, size_t size) = 0;

  BlockWithKey create(size_t size) final;

private:
  BlockWithKey tryCreate(size_t size);
};

}

#endif
