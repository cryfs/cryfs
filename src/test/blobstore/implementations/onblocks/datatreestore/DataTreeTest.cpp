#include "gtest/gtest.h"

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"

#include "fspp/utils/pointer.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;
using fspp::dynamic_pointer_move;

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blockstore::testfake::FakeBlockStore;
using blockstore::BlockStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

class DataTreeTest: public Test {
public:
  DataTreeTest():
    nodeStore(make_unique<FakeBlockStore>()) {
  }

  unique_ptr<DataTree> CreateLeafOnlyTree() {
    auto leafnode = nodeStore.createNewLeafNode();
    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  Key CreateTreeAddOneLeafReturnRootKey() {
    auto tree = CreateLeafOnlyTree();
    auto key = tree->key();
    tree->addDataLeaf();
    return key;
  }

  Key CreateTreeAddTwoLeavesReturnRootKey() {
    auto tree = CreateLeafOnlyTree();
    auto key = tree->key();
    tree->addDataLeaf();
    tree->addDataLeaf();
    return key;
  }

  Key CreateTreeAddThreeLeavesReturnRootKey() {
    auto tree = CreateLeafOnlyTree();
    auto key = tree->key();
    tree->addDataLeaf();
    tree->addDataLeaf();
    tree->addDataLeaf();
    return key;
  }

  Key CreateThreeNodeChainedTreeReturnRootKey() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    auto root = nodeStore.createNewInnerNode(*node);
    return root->key();
  }

  void AddLeafTo(const Key &key) {
    DataTree tree(&nodeStore, nodeStore.load(key));
    tree.addDataLeaf();
  }

  unique_ptr<DataInnerNode> LoadInnerNode(const Key &key) {
    auto node = nodeStore.load(key);
    return dynamic_pointer_move<DataInnerNode>(node);
  }

  unique_ptr<DataLeafNode> LoadLeafNode(const Key &key) {
    auto node = nodeStore.load(key);
    return dynamic_pointer_move<DataLeafNode>(node);
  }

  void EXPECT_IS_LEAF_NODE(const Key &key) {
    auto node = LoadLeafNode(key);
    EXPECT_NE(nullptr, node.get());
  }

  void EXPECT_IS_INNER_NODE(const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_NE(nullptr, node.get());
  }

  DataNodeStore nodeStore;
};

TEST_F(DataTreeTest, GrowAOneNodeTree_KeyDoesntChange) {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  EXPECT_EQ(key, tree->key());
}

TEST_F(DataTreeTest, GrowAOneNodeTree_Structure) {
  auto key = CreateTreeAddOneLeafReturnRootKey();

  EXPECT_IS_INNER_NODE(key);
  auto root = LoadInnerNode(key);

  EXPECT_EQ(2u, root->numChildren());
  EXPECT_IS_LEAF_NODE(root->getChild(0)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(1)->key());
}

TEST_F(DataTreeTest, GrowATwoNodeTree_KeyDoesntChange) {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  tree->addDataLeaf();
  EXPECT_EQ(key, tree->key());
}

TEST_F(DataTreeTest, GrowATwoNodeTree_Structure) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();

  EXPECT_IS_INNER_NODE(key);
  auto root = LoadInnerNode(key);

  EXPECT_EQ(3u, root->numChildren());
  EXPECT_IS_LEAF_NODE(root->getChild(0)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(1)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(2)->key());
}

TEST_F(DataTreeTest, GrowAThreeNodeTree_KeyDoesntChange) {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  tree->addDataLeaf();
  tree->addDataLeaf();
  EXPECT_EQ(key, tree->key());
}

TEST_F(DataTreeTest, GrowAThreeNodeTree_Structure) {
  auto key = CreateTreeAddThreeLeavesReturnRootKey();

  EXPECT_IS_INNER_NODE(key);
  auto root = LoadInnerNode(key);

  EXPECT_EQ(4u, root->numChildren());
  EXPECT_IS_LEAF_NODE(root->getChild(0)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(1)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(2)->key());
  EXPECT_IS_LEAF_NODE(root->getChild(3)->key());
}

TEST_F(DataTreeTest, GrowAThreeNodeChainedTree_KeyDoesntChange) {
  auto root_key = CreateThreeNodeChainedTreeReturnRootKey();
  DataTree tree(&nodeStore, nodeStore.load(root_key));
  tree.addDataLeaf();
  EXPECT_EQ(root_key, tree.key());
}

TEST_F(DataTreeTest, GrowAThreeNodeChainedTree_Structure) {
  auto key = CreateThreeNodeChainedTreeReturnRootKey();
  AddLeafTo(key);

  EXPECT_IS_INNER_NODE(key);
  auto root = LoadInnerNode(key);

  EXPECT_EQ(1u, root->numChildren());
  EXPECT_IS_INNER_NODE(root->getChild(0)->key());
  auto node = LoadInnerNode(root->getChild(0)->key());

  EXPECT_EQ(2u, node->numChildren());
  EXPECT_IS_LEAF_NODE(node->getChild(0)->key());
  EXPECT_IS_LEAF_NODE(node->getChild(1)->key());
}

//TODO Grow a full two-level tree
//TODO Grow a three-level tree
//TODO Go through some cases where the right border node chosen is special
//TODO Test that when growing, the original leaf retains its data

}
}
}
