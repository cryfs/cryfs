#pragma once
#ifndef BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_LEAFDATAFIXTURE_H_
#define BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_LEAFDATAFIXTURE_H_

#include <google/gtest/gtest.h>

#include "../../../../testutils/DataBlockFixture.h"

// A data fixture containing data for a leaf.
// The class can fill this data into a given leaf
// and check, whether the data stored in a given leaf is correct.
class LeafDataFixture {
public:
  LeafDataFixture(int size, int iv = 0): _data(size, iv) {}

  void FillInto(blobstore::onblocks::datanodestore::DataLeafNode *leaf) const {
    leaf->resize(_data.size());
    std::memcpy(leaf->data(), _data.data(), _data.size());
  }

  void EXPECT_DATA_CORRECT(const blobstore::onblocks::datanodestore::DataLeafNode &leaf) const {
    EXPECT_EQ(_data.size(), leaf.numBytes());
    EXPECT_EQ(0, std::memcmp(_data.data(), leaf.data(), _data.size()));
  }

private:
  DataBlockFixture _data;
};

#endif
