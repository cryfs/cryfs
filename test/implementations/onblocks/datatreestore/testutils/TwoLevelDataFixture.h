#pragma once
#ifndef BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_
#define BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_

#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/pointer.h>
#include "LeafDataFixture.h"

//TODO Rename, since we now allow any number of levels
// A data fixture containing data for a two-level tree (one inner node with leaf children).
// The class can fill this data into the leaf children of a given inner node
// and given an inner node can check, whether the data stored is correct.
class TwoLevelDataFixture {
public:
  TwoLevelDataFixture(blobstore::onblocks::datanodestore::DataNodeStore *dataNodeStore, int iv=0, bool useFullSizeLeaves = false): _dataNodeStore(dataNodeStore), _iv(iv), _useFullSizeLeaves(useFullSizeLeaves) {}

  void FillInto(blobstore::onblocks::datanodestore::DataNode *node) {
    // _iv-1 means there is no endLeafIndex - we fill all leaves.
    ForEachLeaf(node, _iv, _iv-1, [this] (blobstore::onblocks::datanodestore::DataLeafNode *leaf, int leafIndex) {
      LeafDataFixture(size(leafIndex), leafIndex).FillInto(leaf);
    });
  }

  void EXPECT_DATA_CORRECT(blobstore::onblocks::datanodestore::DataNode *node, int maxCheckedLeaves = 0) {
    ForEachLeaf(node, _iv, _iv+maxCheckedLeaves, [this] (blobstore::onblocks::datanodestore::DataLeafNode *leaf, int leafIndex) {
      LeafDataFixture(size(leafIndex), leafIndex).EXPECT_DATA_CORRECT(*leaf);
    });
  }

private:
  int ForEachLeaf(blobstore::onblocks::datanodestore::DataNode *node, int firstLeafIndex, int endLeafIndex, std::function<void (blobstore::onblocks::datanodestore::DataLeafNode*, int)> action) {
    if (firstLeafIndex == endLeafIndex) {
      return firstLeafIndex;
    }
    auto leaf = dynamic_cast<blobstore::onblocks::datanodestore::DataLeafNode*>(node);
    if (leaf != nullptr) {
      action(leaf, firstLeafIndex);
      return firstLeafIndex + 1;
    } else {
      auto inner = dynamic_cast<blobstore::onblocks::datanodestore::DataInnerNode*>(node);
      int leafIndex = firstLeafIndex;
      for (int i = 0; i < inner->numChildren(); ++i) {
        auto child = _dataNodeStore->load(inner->getChild(i)->key());
        leafIndex = ForEachLeaf(child.get(), leafIndex, endLeafIndex, action);
      }
      return leafIndex;
    }
  }

  blobstore::onblocks::datanodestore::DataNodeStore *_dataNodeStore;
  int _iv;
  bool _useFullSizeLeaves;

  int size(int childIndex) {
    if (_useFullSizeLeaves) {
      return _dataNodeStore->layout().maxBytesPerLeaf();
    } else {
      uint32_t maxBytesPerLeaf = _dataNodeStore->layout().maxBytesPerLeaf();
      return mod(maxBytesPerLeaf - childIndex, maxBytesPerLeaf);
    }
  }

  int mod(int value, int mod) {
    while(value < 0) {
      value += mod;
    }
    return value;
  }
};

#endif
