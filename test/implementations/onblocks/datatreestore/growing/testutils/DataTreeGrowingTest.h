#pragma once
#ifndef BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_DATATREEGROWINGTEST_H_
#define BLOCKS_MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_DATATREEGROWINGTEST_H_

#include "../../testutils/DataTreeTest.h"

#include "../../../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../../../implementations/onblocks/datanodestore/DataInnerNode.h"

#include "messmer/cpp-utils/pointer.h"

class DataTreeGrowingTest: public DataTreeTest {
public:

  blockstore::Key CreateTreeAddOneLeafReturnRootKey();
  blockstore::Key CreateTreeAddTwoLeavesReturnRootKey();
  blockstore::Key CreateTreeAddThreeLeavesReturnRootKey();
  blockstore::Key CreateThreeNodeChainedTreeReturnRootKey();
  blockstore::Key CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  blockstore::Key CreateThreeLevelTreeWithTwoFullSubtrees();
  void AddLeafTo(const blockstore::Key &key);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> LoadInnerNode(const blockstore::Key &key);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataLeafNode> LoadLeafNode(const blockstore::Key &key);

  void EXPECT_IS_LEAF_NODE(const blockstore::Key &key);
  void EXPECT_IS_INNER_NODE(const blockstore::Key &key);
  void EXPECT_IS_FULL_TWOLEVEL_TREE(const blockstore::Key &key);
  void EXPECT_IS_FULL_THREELEVEL_TREE(const blockstore::Key &key);
  void EXPECT_IS_TWONODE_CHAIN(const blockstore::Key &key);
  void EXPECT_IS_THREENODE_CHAIN(const blockstore::Key &key);
  void EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(const blockstore::Key &key);
  void EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(unsigned int expectedNumberOfLeaves, const blockstore::Key &key);
};


#endif
