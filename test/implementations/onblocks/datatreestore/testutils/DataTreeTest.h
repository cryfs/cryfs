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

  std::unique_ptr<blobstore::onblocks::datatreestore::DataTree> CreateLeafOnlyTree();
  void FillNode(blobstore::onblocks::datanodestore::DataInnerNode *node);
  void FillNodeTwoLevel(blobstore::onblocks::datanodestore::DataInnerNode *node);
  blockstore::Key CreateFullTwoLevelTree();
  blockstore::Key CreateFullThreeLevelTree();
  blobstore::onblocks::datanodestore::DataNodeStore nodeStore;
};


#endif
