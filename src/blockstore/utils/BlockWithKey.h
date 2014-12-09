#pragma once
#ifndef BLOCKSTORE_INTERFACE_BLOCKWITHKEY_H_
#define BLOCKSTORE_INTERFACE_BLOCKWITHKEY_H_

#include <blockstore/interface/Block.h>
#include <memory>
#include "fspp/utils/macros.h"
#include "Key.h"

namespace blockstore {

struct BlockWithKey {
  BlockWithKey(const Key &key_, std::unique_ptr<Block> block_): key(key_), block(std::move(block_)) {}

  Key key;
  std::unique_ptr<Block> block;
};

}

#endif
