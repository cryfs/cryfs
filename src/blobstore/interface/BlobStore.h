#pragma once
#ifndef FSPP_BLOBSTORE_BLOBSTORE_H_
#define FSPP_BLOBSTORE_BLOBSTORE_H_

#include <string>
#include <memory>

namespace blobstore {
class Blob;

//TODO Don't use string, but own class for keys? (better performance for all keys have same length)

class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual std::unique_ptr<Blob> create(const std::string &key, size_t size) = 0;
  virtual std::unique_ptr<Blob> load(const std::string &key) = 0;
  //TODO Needed for performance? Or is deleting loaded blobs enough?
  //virtual void remove(const std::string &key) = 0;
};

}

#endif
