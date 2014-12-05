#pragma once
#ifndef BLOBSTORE_INTERFACE_BLOB_H_
#define BLOBSTORE_INTERFACE_BLOB_H_

namespace blobstore {

class Blob {
public:
  virtual ~Blob() {}

  virtual void *data() = 0;
  virtual const void *data() const = 0;
};

}


#endif
