#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <unordered_map>

namespace blockstore {
namespace inmemory {

class InMemoryBlockStore2 final: public BlockStore2 {
public:
  InMemoryBlockStore2();

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override;
  boost::future<bool> remove(const Key &key) override;
  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override;
  boost::future<void> store(const Key &key, const cpputils::Data &data) override;

private:
  std::unordered_map<Key, cpputils::Data> _blocks;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlockStore2);
};

}
}

#endif
