#pragma once
#ifndef MESSMER_BLOCKSTORE_INTERFACE_HELPERS_BLOCKSTOREWITHRANDOMKEYS_H_
#define MESSMER_BLOCKSTORE_INTERFACE_HELPERS_BLOCKSTOREWITHRANDOMKEYS_H_

#include "../BlockStore.h"
#include "../Block.h"
#include <cpp-utils/random/Random.h>

namespace blockstore {

// This is an implementation helpers for BlockStores that use random block ids.
// You should never give this static type to the client. The client should always
// work with the BlockStore interface instead.
// TODO Delete this class
class BlockStoreWithRandomKeys: public BlockStore {
public:
  BlockId createBlockId() final {
    return BlockId::Random();
  }
};

}

#endif
