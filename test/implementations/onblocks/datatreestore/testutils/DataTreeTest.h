#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_

#include "google/gtest/gtest.h"

#include "../../../../../implementations/onblocks/datanodestore/DataNodeStore.h"
#include "../../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "../../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../../implementations/onblocks/datatreestore/DataTree.h"

class DataTreeTest: public ::testing::Test {
public:
  DataTreeTest();

  std::unique_ptr<blobstore::onblocks::datanodestore::DataLeafNode> CreateLeaf();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::vector<const blobstore::onblocks::datanodestore::DataNode *> children);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::initializer_list<const blobstore::onblocks::datanodestore::DataNode *> children);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateInner(std::initializer_list<std::unique_ptr<blobstore::onblocks::datanodestore::DataNode>> children);

  std::unique_ptr<blobstore::onblocks::datatreestore::DataTree> CreateLeafOnlyTree();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateTwoLeaf();
  std::unique_ptr<blobstore::onblocks::datatreestore::DataTree> CreateTwoLeafTree();
  void FillNode(blobstore::onblocks::datanodestore::DataInnerNode *node);
  void FillNodeTwoLevel(blobstore::onblocks::datanodestore::DataInnerNode *node);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullTwoLevel();
  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> CreateFullThreeLevel();
  blobstore::onblocks::datanodestore::DataNodeStore nodeStore;

  std::unique_ptr<blobstore::onblocks::datanodestore::DataInnerNode> LoadInnerNode(const blockstore::Key &key);
  std::unique_ptr<blobstore::onblocks::datanodestore::DataLeafNode> LoadLeafNode(const blockstore::Key &key);

  void EXPECT_IS_LEAF_NODE(const blockstore::Key &key);
  void EXPECT_IS_INNER_NODE(const blockstore::Key &key);
  void EXPECT_IS_TWONODE_CHAIN(const blockstore::Key &key);
  void EXPECT_IS_FULL_TWOLEVEL_TREE(const blockstore::Key &key);
  void EXPECT_IS_FULL_THREELEVEL_TREE(const blockstore::Key &key);

  void CHECK_DEPTH(int depth, const blockstore::Key &key);
};


#endif
