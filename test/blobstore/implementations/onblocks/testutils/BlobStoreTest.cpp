#include "BlobStoreTest.h"

#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <cstdint>

using blobstore::onblocks::BlobStoreOnBlocks;
using blockstore::testfake::FakeBlockStore;
using cpputils::make_unique_ref;

constexpr uint32_t BlobStoreTest::BLOCKSIZE_BYTES;

BlobStoreTest::BlobStoreTest()
  : blobStore(make_unique_ref<BlobStoreOnBlocks>(make_unique_ref<FakeBlockStore>(), BLOCKSIZE_BYTES)) {
}
