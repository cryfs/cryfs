#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_

#include "../../interface/Blob.h"

#include <memory>

namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTree;
}

class BlobOnBlocks: public Blob {
public:
  BlobOnBlocks(std::unique_ptr<datatreestore::DataTree> datatree);
  virtual ~BlobOnBlocks();

  uint64_t size() const override;
  void resize(uint64_t numBytes) override;

  void flush() const override;

private:
  std::unique_ptr<datatreestore::DataTree> _datatree;
};

}
}

#endif
