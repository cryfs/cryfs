#pragma once
#ifndef BLOBSTORE_INTERFACE_BLOB_H_
#define BLOBSTORE_INTERFACE_BLOB_H_

#include <cstring>
#include <cstdint>

namespace blobstore {

class Blob {
public:
  virtual ~Blob() {}

  virtual uint64_t size() const = 0;
  virtual void resize(uint64_t numBytes) = 0;

  virtual void flush() const = 0;
};

}


#endif
