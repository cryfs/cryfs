#pragma once
#ifndef BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_DATATREESHRINKINGTEST_H_
#define BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_DATATREESHRINKINGTEST_H_

#include "../../testutils/DataTreeTest.h"

#include "../../../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../../../implementations/onblocks/datanodestore/DataInnerNode.h"

#include "messmer/cpp-utils/pointer.h"

class DataTreeShrinkingTest: public DataTreeTest {
public:
  void Shrink(const blockstore::Key &key);

  blockstore::Key CreateFourNodeThreeLeafTree();
  blockstore::Key CreateTwoInnerNodeOneTwoLeavesTree();
  blockstore::Key CreateTwoInnerNodeTwoOneLeavesTree();
  blockstore::Key CreateThreeLevelMinDataTree();
  blockstore::Key CreateFourLevelMinDataTree();
  blockstore::Key CreateFourLevelTreeWithTwoSiblingLeaves1();
  blockstore::Key CreateFourLevelTreeWithTwoSiblingLeaves2();
  blockstore::Key CreateTreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel();
  blockstore::Key CreateThreeLevelTreeWithThreeChildrenOfRoot();
};


#endif
