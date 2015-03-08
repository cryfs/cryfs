#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTORE_H_
#define FSPP_BLOCKSTORE_BLOCKSTORE_H_

#include <messmer/blockstore/interface/Block.h>
#include <string>
#include <memory>


namespace blockstore {

class BlockStore {
public:
  virtual ~BlockStore() {}

  virtual std::unique_ptr<Block> create(size_t size) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual std::unique_ptr<Block> load(const Key &key) = 0;
  virtual void remove(std::unique_ptr<Block> block) = 0;
  virtual uint64_t numBlocks() const = 0;
};

}

#endif
