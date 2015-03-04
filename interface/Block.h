#pragma once
#ifndef BLOCKSTORE_INTERFACE_BLOCK_H_
#define BLOCKSTORE_INTERFACE_BLOCK_H_

#include <messmer/blockstore/utils/Key.h>
#include <cstring>

namespace blockstore {

class Block {
public:
  virtual ~Block() {}

  virtual const void *data() const = 0;
  virtual void write(const void *source, uint64_t offset, uint64_t size) = 0;

  virtual void flush() = 0;

  virtual size_t size() const = 0;

  const Key &key() const {
    return _key;
  }

protected:
  Block(const Key &key) : _key(key) {}

private:
  const Key _key;
};

}


#endif
