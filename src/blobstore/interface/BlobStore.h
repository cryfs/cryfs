#pragma once
#ifndef MESSMER_BLOBSTORE_INTERFACE_BLOBSTORE_H_
#define MESSMER_BLOBSTORE_INTERFACE_BLOBSTORE_H_

#include "Blob.h"
#include <string>
#include <memory>

#include <blockstore/utils/Key.h>
#include <cpp-utils/pointer/unique_ref.h>

namespace blobstore {

class BlobStore {
public:
  virtual ~BlobStore() {}

  virtual cpputils::unique_ref<Blob> create() = 0;
  virtual boost::optional<cpputils::unique_ref<Blob>> load(const blockstore::Key &key) = 0;
  virtual void remove(cpputils::unique_ref<Blob> blob) = 0;
};

}

#endif
