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

  virtual Key createKey() = 0;
  //Returns boost::none if key already exists
  virtual boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual boost::optional<cpputils::unique_ref<Block>> load(const Key &key) = 0;
  virtual void remove(cpputils::unique_ref<Block> block) = 0;
  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes in all block stores
  virtual uint64_t estimateNumFreeBytes() const = 0;

  // Returns, how much space a block has if we allow it to take the given physical block size (i.e. after removing headers, checksums, whatever else).
  // This can be used to create blocks with a certain physical block size.
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0;

  cpputils::unique_ref<Block> create(const cpputils::Data &data) {
    while(true) {
      //TODO Copy (data.copy()) necessary?
      auto block = tryCreate(createKey(), data.copy());
      if (block != boost::none) {
        return std::move(*block);
      }
    }
  }
};

}

#endif
