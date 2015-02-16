#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREETEST_H_

#include "google/gtest/gtest.h"

#include "../../../../implementations/onblocks/datanodestore/DataNodeStore.h"
#include "../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../implementations/onblocks/datatreestore/DataTree.h"
#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"

#include <memory>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using std::make_unique;
using std::unique_ptr;

class DataTreeTest: public ::testing::Test {
public:
  DataTreeTest():
    nodeStore(make_unique<FakeBlockStore>()) {
  }

  unique_ptr<DataTree> CreateLeafOnlyTree() {
    auto leafnode = nodeStore.createNewLeafNode();
    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  void FillNode(DataInnerNode *node) {
    for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
      node->addChild(*nodeStore.createNewLeafNode());
    }
  }

  void FillNodeTwoLevel(DataInnerNode *node) {
    for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
      auto inner_node = nodeStore.createNewInnerNode(*nodeStore.createNewLeafNode());
      for(unsigned int j = 1;j < DataInnerNode::MAX_STORED_CHILDREN; ++j) {
        inner_node->addChild(*nodeStore.createNewLeafNode());
      }
      node->addChild(*inner_node);
    }
  }

  Key CreateFullTwoLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto root = nodeStore.createNewInnerNode(*leaf);
    FillNode(root.get());
    return root->key();
  }

  Key CreateFullThreeLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    auto root = nodeStore.createNewInnerNode(*node);
    FillNode(node.get());
    FillNodeTwoLevel(root.get());
    return root->key();
  }

  DataNodeStore nodeStore;
};


#endif
