#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTORE_H_
#define FSPP_BLOCKSTORE_BLOCKSTORE_H_

#include "Block.h"
#include <string>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Data.h>

namespace blockstore {

class BlockStore {
public:
  virtual ~BlockStore() {}

  virtual Key createKey() = 0;
  //Returns boost::none if key already exists
  virtual boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual boost::optional<cpputils::unique_ref<Block>> load(const Key &key) = 0;
  virtual void remove(cpputils::unique_ref<Block> block) = 0;
  virtual uint64_t numBlocks() const = 0;

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
