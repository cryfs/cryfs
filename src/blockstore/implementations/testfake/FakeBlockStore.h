#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_TESTFAKE_FAKEBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_TESTFAKE_FAKEBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/data/Data.h>
#include <cpp-utils/macros.h>

#include <mutex>
#include <unordered_map>

namespace blockstore {
namespace testfake {
class FakeBlock;

/**
 * This blockstore is meant to be used for unit tests when the module under test needs a blockstore to work with.
 * It basically is the same as the InMemoryBlockStore, but much less forgiving for programming mistakes.
 *
 * InMemoryBlockStore for example simply ignores flushing and gives you access to the same data region each time
 * you request a block. This is very performant, but also forgiving to mistakes. Say you write over the boundaries
 * of a block, then you wouldn't notice, since the next time you access the block, the overflow data is (probably)
 * still there. Or say an application is relying on flushing the block store in the right moment. Since flushing
 * is a no-op in InMemoryBlockStore, you wouldn't notice either.
 *
 * So this FakeBlockStore has a background copy of each block. When you request a block, you will get a copy of
 * the data (instead of a direct pointer as InMemoryBlockStore does) and flushing will copy the data back to the
 * background. This way, tests are more likely to fail if they use the blockstore wrongly.
 */
class FakeBlockStore final: public BlockStore {
public:
  FakeBlockStore();

  BlockId createBlockId() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override;
  cpputils::unique_ref<Block> overwrite(const blockstore::BlockId &blockId, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override;
  void remove(const BlockId &blockId) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

  void updateData(const BlockId &blockId, const cpputils::Data &data);

  void flushBlock(Block* block) override;

private:
  std::unordered_map<BlockId, cpputils::Data> _blocks;

  //This vector keeps a handle of the data regions for all created FakeBlock objects.
  //This way, it is ensured that no two created FakeBlock objects will work on the
  //same data region. Without this, it could happen that a test case creates a FakeBlock,
  //destructs it, creates another one, and the new one gets the same memory region.
  //We want to avoid this for the reasons mentioned above (overflow data).
  std::vector<std::shared_ptr<cpputils::Data>> _used_dataregions_for_blocks;

  mutable std::mutex _mutex;

  cpputils::unique_ref<Block> makeFakeBlockFromData(const BlockId &blockId, const cpputils::Data &data, bool dirty);
  boost::optional<cpputils::unique_ref<Block>> _load(const BlockId &blockId);

  DISALLOW_COPY_AND_ASSIGN(FakeBlockStore);
};

}
}

#endif
