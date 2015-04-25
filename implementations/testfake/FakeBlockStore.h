#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_

#include "../../interface/helpers/BlockStoreWithRandomKeys.h"
#include <messmer/cpp-utils/data/Data.h>
#include "messmer/cpp-utils/macros.h"

#include <mutex>
#include <map>

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
class FakeBlockStore: public BlockStoreWithRandomKeys {
public:
  FakeBlockStore();

  std::unique_ptr<Block> tryCreate(const Key &key, cpputils::Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

  void updateData(const Key &key, const cpputils::Data &data);

private:
  std::map<std::string, cpputils::Data> _blocks;

  //This vector keeps a handle of the data regions for all created FakeBlock objects.
  //This way, it is ensured that no two created FakeBlock objects will work on the
  //same data region. Without this, it could happen that a test case creates a FakeBlock,
  //destructs it, creates another one, and the new one gets the same memory region.
  //We want to avoid this for the reasons mentioned above (overflow data).
  std::vector<std::shared_ptr<cpputils::Data>> _used_dataregions_for_blocks;

  std::unique_ptr<Block> makeFakeBlockFromData(const Key &key, const cpputils::Data &data, bool dirty);

  DISALLOW_COPY_AND_ASSIGN(FakeBlockStore);
};

}
}

#endif
