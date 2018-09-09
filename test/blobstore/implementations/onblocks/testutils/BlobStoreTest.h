#pragma once
#ifndef MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_TESTUTILS_BLOBSTORETEST_H_
#define MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_TESTUTILS_BLOBSTORETEST_H_

#include <gtest/gtest.h>

#include "blobstore/interface/BlobStore.h"

class BlobStoreTest: public ::testing::Test {
public:
  BlobStoreTest();

  static constexpr uint32_t BLOCKSIZE_BYTES = 4096;

  cpputils::unique_ref<blobstore::BlobStore> blobStore;

  cpputils::unique_ref<blobstore::Blob> loadBlob(const blockstore::BlockId &blockId) {
    auto loaded = blobStore->load(blockId);
    EXPECT_TRUE(static_cast<bool>(loaded));
    return std::move(*loaded);
  }

  void reset(cpputils::unique_ref<blobstore::Blob> ref) {
    UNUSED(ref);
    //ref is moved into here and then destructed
  }
};

#endif
