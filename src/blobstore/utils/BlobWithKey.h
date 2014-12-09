#pragma once
#ifndef BLOBSTORE_INTERFACE_BLOBWITHKEY_H_
#define BLOBSTORE_INTERFACE_BLOBWITHKEY_H_

#include <blobstore/interface/Blob.h>
#include <memory>
#include "fspp/utils/macros.h"

namespace blobstore {

struct BlobWithKey {
  BlobWithKey(const std::string &key_, std::unique_ptr<Blob> blob_): key(key_), blob(std::move(blob_)) {}

  std::string key;
  std::unique_ptr<Blob> blob;
};

}

#endif
