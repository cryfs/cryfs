#include "DataTreeTest.h"

#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using std::make_unique;
using std::unique_ptr;

DataTreeTest::DataTreeTest()
  :nodeStore(make_unique<FakeBlockStore>()) {
}

unique_ptr<DataTree> DataTreeTest::CreateLeafOnlyTree() {
  auto leafnode = nodeStore.createNewLeafNode();
  return make_unique<DataTree>(&nodeStore, std::move(leafnode));
}

void DataTreeTest::FillNode(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    node->addChild(*nodeStore.createNewLeafNode());
  }
}

void DataTreeTest::FillNodeTwoLevel(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto inner_node = nodeStore.createNewInnerNode(*nodeStore.createNewLeafNode());
    for(unsigned int j = 1;j < DataInnerNode::MAX_STORED_CHILDREN; ++j) {
      inner_node->addChild(*nodeStore.createNewLeafNode());
    }
    node->addChild(*inner_node);
  }
}

Key DataTreeTest::CreateFullTwoLevelTree() {
  auto leaf = nodeStore.createNewLeafNode();
  auto root = nodeStore.createNewInnerNode(*leaf);
  FillNode(root.get());
  return root->key();
}

Key DataTreeTest::CreateFullThreeLevelTree() {
  auto leaf = nodeStore.createNewLeafNode();
  auto node = nodeStore.createNewInnerNode(*leaf);
  auto root = nodeStore.createNewInnerNode(*node);
  FillNode(node.get());
  FillNodeTwoLevel(root.get());
  return root->key();
}
