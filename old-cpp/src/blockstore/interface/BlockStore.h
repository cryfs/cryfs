#pragma once
#ifndef MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE_H_

#include "Block.h"
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Data.h>

namespace blockstore {

class BlockStore {
public:
  virtual ~BlockStore() {}

  virtual BlockId createBlockId() = 0;
  //Returns boost::none if id already exists
  // TODO Can we make data passed in by ref?
  virtual boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) = 0;
  //TODO Use boost::optional (if id doesn't exist)
  // Return nullptr if block with this id doesn't exists
  virtual boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) = 0;
  virtual cpputils::unique_ref<Block> overwrite(const blockstore::BlockId &blockId, cpputils::Data data) = 0;
  virtual void remove(const BlockId &blockId) = 0;
  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes in all block stores
  virtual uint64_t estimateNumFreeBytes() const = 0;

  // Returns, how much space a block has if we allow it to take the given physical block size (i.e. after removing headers, checksums, whatever else).
  // This can be used to create blocks with a certain physical block size.
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0; // TODO Test

  virtual void forEachBlock(std::function<void (const BlockId &)> callback) const = 0;

  virtual void remove(cpputils::unique_ref<Block> block) {
    BlockId blockId = block->blockId();
    cpputils::destruct(std::move(block));
    remove(blockId);
  }

  cpputils::unique_ref<Block> create(const cpputils::Data &data) {
    while(true) {
      //TODO Copy (data.copy()) necessary?
      auto block = tryCreate(createBlockId(), data.copy());
      if (block != boost::none) {
        return std::move(*block);
      }
    }
  }

  virtual void flushBlock(Block* block) = 0;
};

}

#endif
