#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_THREADSAFE_THREADSAFEBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_THREADSAFE_THREADSAFEBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"

#include <cpp-utils/macros.h>
#include <mutex>
#include <cpp-utils/logging/logging.h>
#include "ThreadsafeBlockStore.h"
#include <cpp-utils/lock/MutexPoolLock.h>

namespace blockstore {
namespace threadsafe {

class ThreadsafeBlock final: public Block {
public:
  ThreadsafeBlock(cpputils::unique_ref<Block> baseBlock, cpputils::MutexPoolLock<BlockId> poolLock);

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

private:
  cpputils::MutexPoolLock<BlockId> _poolLock; // at first position because it should be destructed last
  cpputils::unique_ref<Block> _baseBlock;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(ThreadsafeBlock);
};

}
}

#endif
