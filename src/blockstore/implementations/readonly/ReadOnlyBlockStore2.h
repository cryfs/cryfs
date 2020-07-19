#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_READONLYBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_READONLYBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>

namespace blockstore {
namespace readonly {

// TODO Test

/**
 * Wraps another block store and makes it read-only.
 * All read operations are passed through to the underlying
 * blockstore, while all write operations just throw
 * an exception. This can be used to protect a blockstore
 * if we're in a mode that's supposed to be read-only,
 * e.g. recovery after data corruption.
 */
class ReadOnlyBlockStore2 final: public BlockStore2 {
public:
  ReadOnlyBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore);

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  cpputils::unique_ref<BlockStore2> _baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(ReadOnlyBlockStore2);
};

inline ReadOnlyBlockStore2::ReadOnlyBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore)
: _baseBlockStore(std::move(baseBlockStore)) {
}

inline bool ReadOnlyBlockStore2::tryCreate(const BlockId &/*blockId*/, const cpputils::Data &/*data*/) {
    throw std::logic_error("Tried to call tryCreate on a ReadOnlyBlockStore. Writes to the block store aren't allowed.");
}

inline bool ReadOnlyBlockStore2::remove(const BlockId &/*blockId*/) {
  throw std::logic_error("Tried to call remove on a ReadOnlyBlockStore. Writes to the block store aren't allowed.");
}

inline boost::optional<cpputils::Data> ReadOnlyBlockStore2::load(const BlockId &blockId) const {
    return _baseBlockStore->load(blockId);
}

inline void ReadOnlyBlockStore2::store(const BlockId &/*blockId*/, const cpputils::Data &/*data*/) {
  throw std::logic_error("Tried to call store on a ReadOnlyBlockStore. Writes to the block store aren't allowed.");
}

inline uint64_t ReadOnlyBlockStore2::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

inline uint64_t ReadOnlyBlockStore2::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

inline uint64_t ReadOnlyBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

inline void ReadOnlyBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
  return _baseBlockStore->forEachBlock(std::move(callback));
}

}
}

#endif
