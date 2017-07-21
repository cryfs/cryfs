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

  __attribute__((warn_unused_result))
  virtual bool tryCreate(const Key &key, const cpputils::Data &data) = 0;
  __attribute__((warn_unused_result))
  virtual bool remove(const Key &key) = 0;

  __attribute__((warn_unused_result))
  virtual boost::optional<cpputils::Data> load(const Key &key) const = 0;

  // Store the block with the given key. If it doesn't exist, it is created.
  virtual void store(const Key &key, const cpputils::Data &data) = 0;

  Key create(const cpputils::Data& data) {
    Key key = cpputils::Random::PseudoRandom().getFixedSize<Key::BINARY_LENGTH>();
    bool success = tryCreate(key, data);
    if (success) {
      return key;
    } else {
      return create(data);
    }
  }

  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes
  virtual uint64_t estimateNumFreeBytes() const = 0;
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0; // TODO Test
  virtual void forEachBlock(std::function<void (const Key &)> callback) const = 0;
};

}

#endif
