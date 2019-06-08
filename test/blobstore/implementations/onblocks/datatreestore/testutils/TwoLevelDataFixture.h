#pragma once
#ifndef MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_
#define MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_TWOLEVELDATAFIXTURE_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include "LeafDataFixture.h"
#include <cpp-utils/assert/assert.h>

//TODO Rename, since we now allow any number of levels
// A data fixture containing data for a two-level tree (one inner node with leaf children).
// The class can fill this data into the leaf children of a given inner node
// and given an inner node can check, whether the data stored is correct.
class TwoLevelDataFixture {
public:
  enum class SizePolicy {
    Random,
    Full,
    Unchanged
  };
  TwoLevelDataFixture(blobstore::onblocks::datanodestore::DataNodeStore *dataNodeStore, SizePolicy sizePolicy, int iv=0): _dataNodeStore(dataNodeStore), _iv(iv), _sizePolicy(sizePolicy) {}

  void FillInto(blobstore::onblocks::datanodestore::DataNode *node) {
    // _iv-1 means there is no endLeafIndex - we fill all leaves.
    ForEachLeaf(node, _iv, _iv-1, [this] (blobstore::onblocks::datanodestore::DataLeafNode *leaf, int leafIndex) {
      LeafDataFixture(size(leafIndex, leaf), leafIndex).FillInto(leaf);
    });
  }

  void EXPECT_DATA_CORRECT(blobstore::onblocks::datanodestore::DataNode *node, int maxCheckedLeaves = 0, int lastLeafMaxCheckedBytes = -1) {
    ForEachLeaf(node, _iv, _iv+maxCheckedLeaves, [this, maxCheckedLeaves, lastLeafMaxCheckedBytes] (blobstore::onblocks::datanodestore::DataLeafNode *leaf, int leafIndex) {
      if (leafIndex == _iv+maxCheckedLeaves-1) {
        // It is the last leaf
        LeafDataFixture(size(leafIndex, leaf), leafIndex).EXPECT_DATA_CORRECT(*leaf, lastLeafMaxCheckedBytes);
      } else {
        LeafDataFixture(size(leafIndex, leaf), leafIndex).EXPECT_DATA_CORRECT(*leaf);
      }
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
      for (uint32_t i = 0; i < inner->numChildren(); ++i) {
        auto child = _dataNodeStore->load(inner->readChild(i).blockId()).value();
        leafIndex = ForEachLeaf(child.get(), leafIndex, endLeafIndex, action);
      }
      return leafIndex;
    }
  }

  blobstore::onblocks::datanodestore::DataNodeStore *_dataNodeStore;
  int _iv;
  SizePolicy _sizePolicy;

  int size(int childIndex, blobstore::onblocks::datanodestore::DataLeafNode *leaf) {
    switch (_sizePolicy) {
    case SizePolicy::Full:
      return _dataNodeStore->layout().maxBytesPerLeaf();
    case SizePolicy::Random:
      return mod(static_cast<int>(_dataNodeStore->layout().maxBytesPerLeaf() - childIndex), static_cast<int>(_dataNodeStore->layout().maxBytesPerLeaf()));
    case SizePolicy::Unchanged:
      return leaf->numBytes();
    default:
      ASSERT(false, "Unknown size policy");
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
