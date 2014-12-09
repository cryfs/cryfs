#pragma once
#ifndef FSPP_BLOCKSTORE_BLOCKSTORE_H_
#define FSPP_BLOCKSTORE_BLOCKSTORE_H_

#include <blockstore/interface/Block.h>
#include <blockstore/utils/BlockWithKey.h>
#include <string>
#include <memory>


namespace blockstore {

//TODO Don't use string, but own class for keys? (better performance for all keys have same length)

class BlockStore {
public:
  virtual ~BlockStore() {}

  virtual BlockWithKey create(size_t size) = 0;
  //TODO Use boost::optional (if key doesn't exist)
  // Return nullptr if block with this key doesn't exists
  virtual std::unique_ptr<Block> load(const std::string &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blocks enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
