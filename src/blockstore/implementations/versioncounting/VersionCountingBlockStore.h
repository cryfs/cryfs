#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include "VersionCountingBlock.h"
#include "KnownBlockVersions.h"
#include <iostream>

namespace blockstore {
namespace versioncounting {

class VersionCountingBlockStore final: public BlockStore {
public:
  VersionCountingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore, KnownBlockVersions knownBlockVersions);

  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  void remove(cpputils::unique_ref<Block> block) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  KnownBlockVersions _knownBlockVersions;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStore);
};


inline VersionCountingBlockStore::VersionCountingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore, KnownBlockVersions knownBlockVersions)
 : _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(std::move(knownBlockVersions)) {
}

inline Key VersionCountingBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

inline boost::optional<cpputils::unique_ref<Block>> VersionCountingBlockStore::tryCreate(const Key &key, cpputils::Data data) {
  //TODO Easier implementation? This is only so complicated because of the cast VersionCountingBlock -> Block
  auto result = VersionCountingBlock::TryCreateNew(_baseBlockStore.get(), key, std::move(data), &_knownBlockVersions);
  if (result == boost::none) {
    return boost::none;
  }
  return cpputils::unique_ref<Block>(std::move(*result));
}

inline boost::optional<cpputils::unique_ref<Block>> VersionCountingBlockStore::load(const Key &key) {
  auto block = _baseBlockStore->load(key);
  if (block == boost::none) {
    return boost::none;
  }
  return boost::optional<cpputils::unique_ref<Block>>(VersionCountingBlock::TryLoad(std::move(*block), &_knownBlockVersions));
}

inline void VersionCountingBlockStore::remove(cpputils::unique_ref<Block> block) {
  Key key = block->key();
  auto versionCountingBlock = cpputils::dynamic_pointer_move<VersionCountingBlock>(block);
  ASSERT(versionCountingBlock != boost::none, "Block is not an VersionCountingBlock");
  _knownBlockVersions.incrementVersion(key, (*versionCountingBlock)->version());
  auto baseBlock = (*versionCountingBlock)->releaseBlock();
  _baseBlockStore->remove(std::move(baseBlock));
}

inline uint64_t VersionCountingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

inline uint64_t VersionCountingBlockStore::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

inline uint64_t VersionCountingBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return VersionCountingBlock::blockSizeFromPhysicalBlockSize(_baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize));
}

inline void VersionCountingBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
  return _baseBlockStore->forEachBlock(callback);
}

}
}

#endif
