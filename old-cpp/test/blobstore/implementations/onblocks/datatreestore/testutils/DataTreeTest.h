#pragma once
#ifndef MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_
#define MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_

#include <gtest/gtest.h>
#include <blockstore/implementations/testfake/FakeBlockStore.h>

#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTreeStore.h"
#include "blockstore/implementations/mock/MockBlockStore.h"

class DataTreeTest: public ::testing::Test {
public:
  DataTreeTest();

  static constexpr uint32_t BLOCKSIZE_BYTES = 256;

  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataLeafNode> CreateLeaf();
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::vector<const blobstore::onblocks::datanodestore::DataNode *> children);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::initializer_list<const blobstore::onblocks::datanodestore::DataNode *> children);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::initializer_list<cpputils::unique_ref<blobstore::onblocks::datanodestore::DataNode>> children);

  cpputils::unique_ref<blobstore::onblocks::datatreestore::DataTree> CreateLeafOnlyTree();
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateTwoLeaf();
  cpputils::unique_ref<blobstore::onblocks::datatreestore::DataTree> CreateTwoLeafTree();
  void FillNode(blobstore::onblocks::datanodestore::DataInnerNode *node);
  void FillNodeTwoLevel(blobstore::onblocks::datanodestore::DataInnerNode *node);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullTwoLevel();
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullThreeLevel();

  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateThreeLevelMinData();
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFourLevelMinData();

  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> LoadInnerNode(const blockstore::BlockId &blockId);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataLeafNode> LoadLeafNode(const blockstore::BlockId &blockId);

  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataLeafNode> CreateLeafWithSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateTwoLeafWithSecondLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullTwoLevelWithLastLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateThreeLevelWithOneChildAndLastLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateThreeLevelWithTwoChildrenAndLastLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateThreeLevelWithThreeChildrenAndLastLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullThreeLevelWithLastLeafSize(uint32_t size);
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataInnerNode> CreateFourLevelMinDataWithLastLeafSize(uint32_t size);

  cpputils::unique_ref<blockstore::mock::MockBlockStore> _blockStore;
  blockstore::mock::MockBlockStore *blockStore;
  cpputils::unique_ref<blobstore::onblocks::datanodestore::DataNodeStore> _nodeStore;
  blobstore::onblocks::datanodestore::DataNodeStore *nodeStore;
  blobstore::onblocks::datatreestore::DataTreeStore treeStore;

  void EXPECT_IS_LEAF_NODE(const blockstore::BlockId &blockId);
  void EXPECT_IS_INNER_NODE(const blockstore::BlockId &blockId);
  void EXPECT_IS_TWONODE_CHAIN(const blockstore::BlockId &blockId);
  void EXPECT_IS_FULL_TWOLEVEL_TREE(const blockstore::BlockId &blockId);
  void EXPECT_IS_FULL_THREELEVEL_TREE(const blockstore::BlockId &blockId);

  void CHECK_DEPTH(int depth, const blockstore::BlockId &blockId);

private:
  DISALLOW_COPY_AND_ASSIGN(DataTreeTest);
};


#endif
