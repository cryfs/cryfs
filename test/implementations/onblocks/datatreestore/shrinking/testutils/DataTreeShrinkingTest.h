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

  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateFourNodeThreeLeaf();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateTwoInnerNodeOneTwoLeaves();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateTwoInnerNodeTwoOneLeaves();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateFourLevelWithTwoSiblingLeaves1();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateFourLevelWithTwoSiblingLeaves2();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateThreeLevelWithThreeChildrenOfRoot();
};


#endif
