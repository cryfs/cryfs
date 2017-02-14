#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "OnDiskBlockStore.h"

namespace blockstore {
namespace ondisk {

//TODO Implement without basing on OnDiskBlockStore

class OnDiskBlockStore2 final: public BlockStore2 {
public:
  explicit OnDiskBlockStore2(cpputils::unique_ref<OnDiskBlockStore> base)
    : _base(std::move(base)) {}

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override {
    auto created = _base->tryCreate(key, data.copy());
    if (boost::none == created) {
      return boost::make_ready_future(false);
    } else {
      return boost::make_ready_future(true);
    }
  }

  boost::future<bool> remove(const Key &key) override {
    _base->remove(key);
    return boost::make_ready_future(true);
  }

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
    auto block = _base->load(key);
    if (boost::none == block) {
      return boost::make_ready_future<boost::optional<cpputils::Data>>(boost::none);
    }
    cpputils::Data data((*block)->size());
    std::memcpy(data.data(), (*block)->data(), data.size());
    return boost::make_ready_future<boost::optional<cpputils::Data>>(std::move(data));
  }

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    _base->overwrite(key, data.copy());
    return boost::make_ready_future();
  }

private:
  cpputils::unique_ref<OnDiskBlockStore> _base;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore2);
};

}
}

#endif
