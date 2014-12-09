#pragma once
#ifndef BLOBSTORE_INTERFACE_BLOBWITHKEY_H_
#define BLOBSTORE_INTERFACE_BLOBWITHKEY_H_

#include <blobstore/interface/Blob.h>
#include <memory>
#include "fspp/utils/macros.h"
#include "blockstore/utils/Key.h"

namespace blobstore {

//TODO Use own key class to become independent from blockstore?
typedef blockstore::Key Key;

struct BlobWithKey {
  BlobWithKey(const Key &key_, std::unique_ptr<Blob> blob_): key(key_), blob(std::move(blob_)) {}

  Key key;
  std::unique_ptr<Blob> blob;
};

}

#endif
