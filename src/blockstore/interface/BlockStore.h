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
  // Return nullptr if block with this key doesn't exists
  virtual boost::optional<cpputils::unique_ref<Block>> load(const Key &key) = 0;
  virtual cpputils::unique_ref<Block> overwrite(const blockstore::Key &key, cpputils::Data data) = 0;
  virtual void remove(const Key &key) = 0;
  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes in all block stores
  virtual uint64_t estimateNumFreeBytes() const = 0;

  // Returns, how much space a block has if we allow it to take the given physical block size (i.e. after removing headers, checksums, whatever else).
  // This can be used to create blocks with a certain physical block size.
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0;

  virtual void forEachBlock(std::function<void (const Key &)> callback) const = 0;

  // TODO Test exists()
  virtual bool exists(const Key &key) const = 0;

  // TODO Test loadOrCreate()
  // TODO Also test using the block after loadOrCreate(), e.g. writing to it. CachingBlockStore handles these blocks quite different from blocks loaded using load().
  // TODO Implement this per block store? (more efficient, without calling load())
  virtual cpputils::unique_ref<Block> loadOrCreate(const Key &key, size_t size) {
    auto loaded = load(key);
    if (loaded == boost::none) {
      auto created = tryCreate(key, cpputils::Data(size).FillWithZeroes());
      ASSERT(created != boost::none, "Couldn't load and also couldn't create block. One should succeed.");
      return std::move(*created);
    } else {
      ASSERT((*loaded)->size() == size, "Loaded block of different size");
      return std::move(*loaded);
    }
  }

  //TODO Test removeIfExists
  // TODO Implement this per block store? Probably faster.
  virtual void removeIfExists(const Key &key) {
    if (exists(key)) {
      remove(key);
    }
  }

  virtual void remove(cpputils::unique_ref<Block> block) {
    Key key = block->key();
    cpputils::destruct(std::move(block));
    remove(key);
  }

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
