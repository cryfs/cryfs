#pragma once
#ifndef MESSMER_BLOBSTORE_INTERFACE_BLOB_H_
#define MESSMER_BLOBSTORE_INTERFACE_BLOB_H_

#include <cstring>
#include <cstdint>
#include <blockstore/utils/BlockId.h>
#include <cpp-utils/data/Data.h>

namespace blobstore {

class Blob {
public:
  virtual ~Blob() {}

  //TODO Use own Id class for blobstore
  virtual const blockstore::BlockId &blockId() const = 0;

  virtual uint64_t size() const = 0;
  virtual void resize(uint64_t numBytes) = 0;

  virtual cpputils::Data readAll() const = 0;
  virtual void read(void *target, uint64_t offset, uint64_t size) const = 0;
  virtual uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const = 0;
  virtual void write(const void *source, uint64_t offset, uint64_t size) = 0;

  virtual void flush() = 0;

  virtual uint32_t numNodes() const = 0;

  //TODO Test tryRead
};

}


#endif
