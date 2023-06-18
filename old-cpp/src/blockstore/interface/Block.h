#pragma once
#ifndef MESSMER_BLOCKSTORE_INTERFACE_BLOCK_H_
#define MESSMER_BLOCKSTORE_INTERFACE_BLOCK_H_

#include "blockstore/utils/BlockId.h"
#include <cstring>

namespace blockstore {

//TODO Make Block non-virtual class that stores ptr to its blockstore and writes itself back to the blockstore who is offering a corresponding function.
//     Then ondisk blockstore can be actually create the file on disk in blockstore::create() and cachingblockstore will delay that call to its base block store.

class Block {
public:
  virtual ~Block() {}

  virtual const void *data() const = 0;
  virtual void write(const void *source, uint64_t offset, uint64_t size) = 0;

  virtual size_t size() const = 0;

  //TODO Test resize()
  virtual void resize(size_t newSize) = 0;

  const BlockId &blockId() const {
    return _blockId;
  }

protected:
  Block(const BlockId &blockId) : _blockId(blockId) {}

private:
  const BlockId _blockId;
};

}


#endif
