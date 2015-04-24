#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTORE_H_
#define FSPP_BLOCKSTORE_BLOCKSTORE_H_

#include "Block.h"
#include <string>
#include <memory>
#include "../utils/Data.h"

namespace blockstore {

class BlockStore {
public:
  virtual ~BlockStore() {}

  virtual Key createKey() = 0;
  //Returns nullptr if key already exists
  virtual std::unique_ptr<Block> tryCreate(const Key &key, Data data) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual std::unique_ptr<Block> load(const Key &key) = 0;
  virtual void remove(std::unique_ptr<Block> block) = 0;
  virtual uint64_t numBlocks() const = 0;

  std::unique_ptr<Block> create(const Data &data) {
    std::unique_ptr<Block> block(nullptr);
    while(block.get() == nullptr) {
      //TODO Copy necessary?
      block = tryCreate(createKey(), data.copy());
    }
    return block;
  }
};

}

#endif
