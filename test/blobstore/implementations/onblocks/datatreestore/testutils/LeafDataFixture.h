#pragma once
#ifndef MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_LEAFDATAFIXTURE_H_
#define MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_LEAFDATAFIXTURE_H_

#include <gtest/gtest.h>

#include <cpp-utils/data/DataFixture.h>

// A data fixture containing data for a leaf.
// The class can fill this data into a given leaf
// and check, whether the data stored in a given leaf is correct.
class LeafDataFixture {
public:
  LeafDataFixture(int size, int iv = 0): _data(cpputils::DataFixture::generate(size, iv)) {}

  void FillInto(blobstore::onblocks::datanodestore::DataLeafNode *leaf) const {
    leaf->resize(_data.size());
    leaf->write(_data.data(), 0, _data.size());
  }

  void EXPECT_DATA_CORRECT(const blobstore::onblocks::datanodestore::DataLeafNode &leaf, int onlyCheckNumBytes = -1) const {
    if (onlyCheckNumBytes == -1) {
      EXPECT_EQ(_data.size(), leaf.numBytes());
      EXPECT_EQ(0, std::memcmp(_data.data(), loadData(leaf).data(), _data.size()));
    } else {
      EXPECT_LE(onlyCheckNumBytes, static_cast<int>(leaf.numBytes()));
      EXPECT_EQ(0, std::memcmp(_data.data(), loadData(leaf).data(), onlyCheckNumBytes));
    }
  }

private:
  static cpputils::Data loadData(const blobstore::onblocks::datanodestore::DataLeafNode &leaf) {
    cpputils::Data data(leaf.numBytes());
    leaf.read(data.data(), 0, leaf.numBytes());
    return data;
  }
  cpputils::Data _data;
};

#endif
