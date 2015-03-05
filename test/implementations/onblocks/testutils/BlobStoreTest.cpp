#include "BlobStoreTest.h"

#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>
#include "../../../../implementations/onblocks/BlobStoreOnBlocks.h"

using std::make_unique;
using std::unique_ptr;

using blobstore::onblocks::BlobStoreOnBlocks;
using blockstore::testfake::FakeBlockStore;

constexpr uint32_t BlobStoreTest::BLOCKSIZE_BYTES;

BlobStoreTest::BlobStoreTest()
  : blobStore(make_unique<BlobStoreOnBlocks>(make_unique<FakeBlockStore>(), BLOCKSIZE_BYTES)) {
}
