#pragma once
#ifndef MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_INTERFACE_BLOCKSTORE2_H_

#include "Block.h"
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Data.h>
#include <boost/thread/future.hpp>
#include <cpp-utils/random/Random.h>

// TODO warn_unused_result for all boost::future interfaces

namespace blockstore {

class BlockStore2 {
public:
  virtual ~BlockStore2() {}

  __attribute__((warn_unused_result))
  virtual boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) = 0;
  __attribute__((warn_unused_result))
  virtual boost::future<bool> remove(const Key &key) = 0;

  __attribute__((warn_unused_result))
  virtual boost::future<boost::optional<cpputils::Data>> load(const Key &key) const = 0;

  // Store the block with the given key. If it doesn't exist, it is created.
  __attribute__((warn_unused_result))
  virtual boost::future<void> store(const Key &key, const cpputils::Data &data) = 0;

  __attribute__((warn_unused_result))
  boost::future<Key> create(cpputils::Data data) {
    Key key = cpputils::Random::PseudoRandom().getFixedSize<Key::BINARY_LENGTH>();
    boost::future<bool> successFuture = tryCreate(key, data);
    return successFuture.then([this, key, data = std::move(data)] (boost::future<bool> success) mutable {
      if (success.get()) {
        return boost::make_ready_future<Key>(key);
      } else {
        return this->create(std::move(data));
      }
    });
  }

  virtual uint64_t numBlocks() const = 0;
  //TODO Test estimateNumFreeBytes
  virtual uint64_t estimateNumFreeBytes() const = 0;
  virtual uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const = 0; // TODO Test
  virtual void forEachBlock(std::function<void (const Key &)> callback) const = 0;
};

}

#endif
