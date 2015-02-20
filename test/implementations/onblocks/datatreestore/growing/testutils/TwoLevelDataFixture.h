#pragma once
#ifndef BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_
#define BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_

#include <messmer/cpp-utils/macros.h>

// A data fixture containing data for a two-level tree (one inner node with leaf children).
// The class can fill this data into the leaf children of a given inner node
// and given an inner node can check, whether the data stored is correct.
class TwoLevelDataFixture {
public:
  TwoLevelDataFixture(blobstore::onblocks::datanodestore::DataNodeStore *dataNodeStore): _dataNodeStore(dataNodeStore) {}

  void FillInto(blobstore::onblocks::datanodestore::DataInnerNode *node) {
    for (int i = 0; i < node->numChildren(); ++i) {
      auto leafnode = _dataNodeStore->load(node->getChild(i)->key());
      auto leaf = cpputils::dynamic_pointer_move<blobstore::onblocks::datanodestore::DataLeafNode>(leafnode);
      LeafDataFixture(size(i), i).FillInto(leaf.get());
    }
  }

  void EXPECT_DATA_CORRECT(const blobstore::onblocks::datanodestore::DataInnerNode &node) const {
    for (int i = 0; i < node.numChildren(); ++i) {
      auto leafnode =_dataNodeStore->load(node.getChild(i)->key());
      auto leaf = cpputils::dynamic_pointer_move<blobstore::onblocks::datanodestore::DataLeafNode>(leafnode);
      LeafDataFixture(size(i), i).EXPECT_DATA_CORRECT(*leaf);
    }
  }

private:
  blobstore::onblocks::datanodestore::DataNodeStore *_dataNodeStore;

  static int size(int childIndex) {
    return blobstore::onblocks::datanodestore::DataLeafNode::MAX_STORED_BYTES-childIndex;
  }
};

#endif
