#pragma once
#ifndef BLOBSTORE_INTERFACE_BLOB_H_
#define BLOBSTORE_INTERFACE_BLOB_H_

#include <cstring>

namespace blobstore {

class Blob {
public:
  virtual ~Blob() {}

  virtual size_t size() const = 0;

  virtual void flush() const = 0;
};

}


#endif
