#include "BlobStoreTest.h"

#include "blobstore/implementations/rustbridge/RustBlobStore.h"
#include <cpp-utils/pointer/gcc_4_8_compatibility.h>

using cpputils::make_unique_ref;

constexpr uint32_t BlobStoreTest::BLOCKSIZE_BYTES;

BlobStoreTest::BlobStoreTest()
  //: blobStore(make_unique_ref<BlobStoreOnBlocks>(make_unique_ref<FakeBlockStore>(), BLOCKSIZE_BYTES)) {
    : blobStore(make_unique_ref<blobstore::rust::RustBlobStore>(
      blobstore::rust::bridge::new_locking_inmemory_blobstore(BLOCKSIZE_BYTES))) {
}
