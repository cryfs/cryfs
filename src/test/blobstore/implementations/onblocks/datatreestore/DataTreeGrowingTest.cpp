#include "DataTreeTest.h"

#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "test/testutils/DataBlockFixture.h"

#include "fspp/utils/pointer.h"

using fspp::dynamic_pointer_move;

using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blockstore::Key;

using namespace blobstore::onblocks::datatreestore;

class DataTreeGrowingTest: public DataTreeTest {
public:

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

  Key CreateThreeLevelTreeWithLowerLevelFullReturnRootKey() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    FillNode(node.get());
    auto root = nodeStore.createNewInnerNode(*node);
    return root->key();
  }

  Key CreateThreeLevelTreeWithTwoFullSubtrees() {
    auto leaf1 = nodeStore.createNewLeafNode();
    auto leaf2 = nodeStore.createNewLeafNode();
    auto leaf3 = nodeStore.createNewLeafNode();
    auto node1 = nodeStore.createNewInnerNode(*leaf1);
    FillNode(node1.get());
    auto node2 = nodeStore.createNewInnerNode(*leaf2);
    FillNode(node2.get());
    auto root = nodeStore.createNewInnerNode(*node1);
    root->addChild(*node2);
    return root->key();
  }

  void AddLeafTo(const Key &key) {
    DataTree tree(&nodeStore, nodeStore.load(key));
    tree.addDataLeaf();
  }

  unique_ptr<DataInnerNode> LoadInnerNode(const Key &key) {
    auto node = nodeStore.load(key);
    auto casted = dynamic_pointer_move<DataInnerNode>(node);
    EXPECT_NE(nullptr, casted.get()) << "Is not an inner node";
    return casted;
  }

  unique_ptr<DataLeafNode> LoadLeafNode(const Key &key) {
    auto node = nodeStore.load(key);
    auto casted =  dynamic_pointer_move<DataLeafNode>(node);
    EXPECT_NE(nullptr, casted.get()) << "Is not a leaf node";
    return casted;
  }

  void EXPECT_IS_LEAF_NODE(const Key &key) {
    auto node = LoadLeafNode(key);
    EXPECT_NE(nullptr, node.get());
  }

  void EXPECT_IS_INNER_NODE(const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_NE(nullptr, node.get());
  }

  void EXPECT_IS_FULL_TWOLEVEL_TREE(const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, node->numChildren());
    for (unsigned int i = 0; i < node->numChildren(); ++i) {
      EXPECT_IS_LEAF_NODE(node->getChild(i)->key());
    }
  }

  void EXPECT_IS_FULL_THREELEVEL_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, root->numChildren());
    for (unsigned int i = 0; i < root->numChildren(); ++i) {
      auto node = LoadInnerNode(root->getChild(i)->key());
      EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, node->numChildren());
      for (unsigned int j = 0; j < node->numChildren(); ++j) {
        EXPECT_IS_LEAF_NODE(node->getChild(j)->key());
      }
    }
  }

  void EXPECT_IS_TWONODE_CHAIN(const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_EQ(1u, node->numChildren());
    EXPECT_IS_LEAF_NODE(node->getChild(0)->key());
  }

  void EXPECT_IS_THREENODE_CHAIN(const Key &key) {
    auto node1 = LoadInnerNode(key);
    EXPECT_EQ(1u, node1->numChildren());
    auto node2 = LoadInnerNode(node1->getChild(0)->key());
    EXPECT_EQ(1u, node2->numChildren());
    EXPECT_IS_LEAF_NODE(node2->getChild(0)->key());
  }

  void EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(const Key &key) {
    DataTree tree(&nodeStore, nodeStore.load(key));
    tree.addDataLeaf();
    EXPECT_EQ(key, tree.key());
  }

  void EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(unsigned int expectedNumberOfLeaves, const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_EQ(expectedNumberOfLeaves, node->numChildren());
    for(unsigned int i=0;i<expectedNumberOfLeaves;++i) {
      EXPECT_IS_LEAF_NODE(node->getChild(i)->key());
    }
  }
};

TEST_F(DataTreeGrowingTest, GrowAOneNodeTree_KeyDoesntChange) {
  auto key = CreateLeafOnlyTree()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest, GrowAOneNodeTree_Structure) {
  auto key = CreateTreeAddOneLeafReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, key);
}

TEST_F(DataTreeGrowingTest, GrowAOneNodeTree_FlushingWorks) {
  //Tests that after calling flush(), the complete grown tree structure is written to the blockstore
  auto tree = CreateLeafOnlyTree();
  tree->addDataLeaf();
  tree->flush();

  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, tree->key());
}

TEST_F(DataTreeGrowingTest, GrowATwoNodeTree_KeyDoesntChange) {
  auto key = CreateTreeAddOneLeafReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest, GrowATwoNodeTree_Structure) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(3, key);
}

TEST_F(DataTreeGrowingTest, GrowATwoLevelThreeNodeTree_KeyDoesntChange) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest, GrowATwoLevelThreeNodeTree_Structure) {
  auto key = CreateTreeAddThreeLeavesReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(4, key);
}

TEST_F(DataTreeGrowingTest, GrowAThreeNodeChainedTree_KeyDoesntChange) {
  auto root_key = CreateThreeNodeChainedTreeReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest, GrowAThreeNodeChainedTree_Structure) {
  auto key = CreateThreeNodeChainedTreeReturnRootKey();
  AddLeafTo(key);

  auto root = LoadInnerNode(key);
  EXPECT_EQ(1u, root->numChildren());

  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, root->getChild(0)->key());
}

TEST_F(DataTreeGrowingTest, GrowAThreeLevelTreeWithLowerLevelFull_KeyDoesntChange) {
  auto root_key = CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest, GrowAThreeLevelTreeWithLowerLevelFull_Structure) {
  auto root_key = CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest, GrowAFullTwoLevelTree_KeyDoesntChange) {
  auto root_key = CreateFullTwoLevelTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest, GrowAFullTwoLevelTree_Structure) {
  auto root_key = CreateFullTwoLevelTree();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest, GrowAFullThreeLevelTree_KeyDoesntChange) {
  auto root_key = CreateFullThreeLevelTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest, GrowAFullThreeLevelTree_Structure) {
  auto root_key = CreateFullThreeLevelTree();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_THREELEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_THREENODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest, GrowAThreeLevelTreeWithTwoFullSubtrees_KeyDoesntChange) {
  auto root_key = CreateThreeLevelTreeWithTwoFullSubtrees();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest, GrowAThreeLevelTreeWithTwoFullSubtrees_Structure) {
  auto root_key = CreateThreeLevelTreeWithTwoFullSubtrees();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(3u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(1)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(2)->key());
}

class DataTreeGrowingDataTest: public DataTreeGrowingTest {
public:
  DataTreeGrowingDataTest(): data(DataLeafNode::MAX_STORED_BYTES-2) {}
  DataBlockFixture data;

  unique_ptr<DataTree> CreateLeafOnlyTreeWithData() {
    auto leafnode = nodeStore.createNewLeafNode();
    leafnode->resize(data.size());
    std::memcpy(leafnode->data(), data.data(), data.size());

    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  unique_ptr<DataTree> CreateTwoNodeTreeWithData() {
    auto tree = CreateLeafOnlyTreeWithData();
    tree->addDataLeaf();
    return tree;
  }

  unique_ptr<DataTree> CreateThreeNodeChainedTreeWithData() {
    auto leaf = nodeStore.createNewLeafNode();
    leaf->resize(data.size());
    std::memcpy(leaf->data(), data.data(), data.size());

    auto inner = nodeStore.createNewInnerNode(*leaf);
    return make_unique<DataTree>(&nodeStore, nodeStore.createNewInnerNode(*inner));
  }

  unique_ptr<DataLeafNode> LoadFirstLeafOf(const Key &key) {
    auto root = LoadInnerNode(key);
    return LoadLeafNode(root->getChild(0)->key());
  }

  unique_ptr<DataLeafNode> LoadTwoLevelFirstLeafOf(const Key &key) {
    auto root = LoadInnerNode(key);
    auto inner = LoadInnerNode(root->getChild(0)->key());
    return LoadLeafNode(inner->getChild(0)->key());
  }

  void EXPECT_DATA_CORRECT(const DataLeafNode &leaf) {
    EXPECT_EQ(data.size(), leaf.numBytes());
    EXPECT_EQ(0, std::memcmp(data.data(), leaf.data(), data.size()));
  }
};

TEST_F(DataTreeGrowingDataTest, GrowAOneNodeTree_DataStaysIntact) {
  auto tree = CreateLeafOnlyTreeWithData();
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  EXPECT_DATA_CORRECT(*leaf);
}

TEST_F(DataTreeGrowingDataTest, GrowATwoNodeTree_DataStaysIntact) {
  auto tree = CreateTwoNodeTreeWithData();
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  EXPECT_DATA_CORRECT(*leaf);
}

TEST_F(DataTreeGrowingDataTest, GrowAThreeNodeChainedTree_DataStaysIntact) {
  auto tree = CreateThreeNodeChainedTreeWithData();
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadTwoLevelFirstLeafOf(tree->key());
  EXPECT_DATA_CORRECT(*leaf);
}

//TODO Test that when growing, the original leaves retains its data (for ThreeLevelTreeWithLowerLevelFull, FullTwoLevelTree)
//TODO Test tree depth markers on the nodes
//TODO Build-up test cases (build a leaf tree, add N leaves and check end state. End states for example FullTwoLevelTree, FullThreeLevelTree)
