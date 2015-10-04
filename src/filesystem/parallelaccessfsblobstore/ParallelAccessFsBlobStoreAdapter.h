#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTOREADAPTER_H_

#include <messmer/cpp-utils/macros.h>
#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "../fsblobstore/FsBlobStore.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class ParallelAccessFsBlobStoreAdapter: public parallelaccessstore::ParallelAccessBaseStore<fsblobstore::FsBlob, blockstore::Key> {
public:
  explicit ParallelAccessFsBlobStoreAdapter(fsblobstore::FsBlobStore *baseBlockStore)
    :_baseBlockStore(std::move(baseBlockStore)) {
  }

  boost::optional<cpputils::unique_ref<fsblobstore::FsBlob>> loadFromBaseStore(const blockstore::Key &key) override {
	return _baseBlockStore->load(key);
  }

  void removeFromBaseStore(cpputils::unique_ref<fsblobstore::FsBlob> block) override {
	return _baseBlockStore->remove(std::move(block));
  }

private:
  fsblobstore::FsBlobStore *_baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStoreAdapter);
};

}
}

#endif
