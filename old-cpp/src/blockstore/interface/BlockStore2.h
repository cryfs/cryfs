#pragma once
#ifndef MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE2_H_

#include "Block.h"
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Data.h>
#include <cpp-utils/random/Random.h>

namespace blockstore {

class BlockStore2 {
public:
  virtual ~BlockStore2() {}

  virtual BlockId createBlockId() const {
    return BlockId::Random();
  }

  WARN_UNUSED_RESULT
  virtual bool tryCreate(const BlockId &blockId, const cpputils::Data &data) = 0;
  WARN_UNUSED_RESULT
  virtual bool remove(const BlockId &blockId) = 0;

  WARN_UNUSED_RESULT
  virtual boost::optional<cpputils::Data> load(const BlockId &blockId) const = 0;

  // Store the block with the given blockId. If it doesn't exist, it is created.
  virtual void store(const BlockId &blockId, const cpputils::Data &data) = 0;

  BlockId create(const cpputils::Data& data) {
    while (true) {
      BlockId blockId = createBlockId();
      bool success = tryCreate(blockId, data);
      if (success) {
        return blockId;
      }
    }
  }

  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes
  virtual uint64_t estimateNumFreeBytes() const = 0;
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0; // TODO Test
  virtual void forEachBlock(std::function<void (const BlockId &)> callback) const = 0;
};

}

#endif
