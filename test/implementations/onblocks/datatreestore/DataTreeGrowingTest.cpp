#include "DataTreeTest.h"

#include "../../../../implementations/onblocks/datatreestore/DataTree.h"
#include "../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "../../../testutils/DataBlockFixture.h"

#include "messmer/cpp-utils/pointer.h"

using cpputils::dynamic_pointer_move;

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

class LeafDataFixture {
public:
  LeafDataFixture(int size, int iv = 0): _data(size, iv) {}

  void FillInto(DataLeafNode *leaf) const {
    leaf->resize(_data.size());
    std::memcpy(leaf->data(), _data.data(), _data.size());
  }

  void EXPECT_DATA_CORRECT(const DataLeafNode &leaf) const {
    EXPECT_EQ(_data.size(), leaf.numBytes());
    EXPECT_EQ(0, std::memcmp(_data.data(), leaf.data(), _data.size()));
  }

private:
  DataBlockFixture _data;
};

class TwoLevelDataFixture {
public:
  TwoLevelDataFixture(DataNodeStore *dataNodeStore): _dataNodeStore(dataNodeStore) {}

  void FillInto(DataInnerNode *node) {
    for (int i = 0; i < node->numChildren(); ++i) {
      auto leafnode = _dataNodeStore->load(node->getChild(i)->key());
      auto leaf = dynamic_pointer_move<DataLeafNode>(leafnode);
      LeafDataFixture(size(i), i).FillInto(leaf.get());
    }
  }

  void EXPECT_DATA_CORRECT(const DataInnerNode &node) const {
    for (int i = 0; i < node.numChildren(); ++i) {
      auto leafnode =_dataNodeStore->load(node.getChild(i)->key());
      auto leaf = dynamic_pointer_move<DataLeafNode>(leafnode);
      LeafDataFixture(size(i), i).EXPECT_DATA_CORRECT(*leaf);
    }
  }

private:
  DataNodeStore *_dataNodeStore;

  static int size(int childIndex) {
    return DataLeafNode::MAX_STORED_BYTES-childIndex;
  }
};

class DataTreeGrowingDataTest: public DataTreeGrowingTest {
public:
  unique_ptr<DataTree> CreateLeafOnlyTreeWithData(const LeafDataFixture &data) {
    auto leafnode = nodeStore.createNewLeafNode();
    data.FillInto(leafnode.get());

    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  unique_ptr<DataTree> CreateTwoNodeTreeWithData(const LeafDataFixture &data) {
    auto tree = CreateLeafOnlyTreeWithData(data);
    tree->addDataLeaf();
    return tree;
  }

  unique_ptr<DataTree> CreateThreeNodeChainedTreeWithData(const LeafDataFixture &data) {
    auto leaf = nodeStore.createNewLeafNode();
    data.FillInto(leaf.get());

    auto inner = nodeStore.createNewInnerNode(*leaf);
    return make_unique<DataTree>(&nodeStore, nodeStore.createNewInnerNode(*inner));
  }

  unique_ptr<DataTree> CreateFullTwoLevelTreeWithData(TwoLevelDataFixture *data) {
    auto root = LoadInnerNode(CreateFullTwoLevelTree());
    assert(root->numChildren() == DataInnerNode::MAX_STORED_CHILDREN);
    data->FillInto(root.get());
    return make_unique<DataTree>(&nodeStore, std::move(root));
  }

  unique_ptr<DataTree> CreateThreeLevelTreeWithLowerLevelFullWithData(TwoLevelDataFixture *data) {
    auto _node = LoadFirstChildOf(CreateThreeLevelTreeWithLowerLevelFullReturnRootKey());
    auto node = dynamic_pointer_move<DataInnerNode>(_node);
    data->FillInto(node.get());
    return make_unique<DataTree>(&nodeStore, std::move(node));
  }

  unique_ptr<DataNode> LoadFirstChildOf(const Key &key) {
    auto root = LoadInnerNode(key);
    return nodeStore.load(root->getChild(0)->key());
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
};

TEST_F(DataTreeGrowingDataTest, GrowAOneNodeTree_DataStaysIntact) {
  LeafDataFixture data(DataLeafNode::MAX_STORED_BYTES-1);
  auto tree = CreateLeafOnlyTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}

TEST_F(DataTreeGrowingDataTest, GrowATwoNodeTree_DataStaysIntact) {
  LeafDataFixture data(DataLeafNode::MAX_STORED_BYTES-1);
  auto tree = CreateTwoNodeTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}

TEST_F(DataTreeGrowingDataTest, GrowAThreeNodeChainedTree_DataStaysIntact) {
  LeafDataFixture data(DataLeafNode::MAX_STORED_BYTES-1);
  auto tree = CreateThreeNodeChainedTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadTwoLevelFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}

TEST_F(DataTreeGrowingDataTest, GrowAFullTwoLevelTree_DataStaysIntact) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateFullTwoLevelTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto node = LoadFirstChildOf(tree->key());
  data.EXPECT_DATA_CORRECT(*dynamic_pointer_move<DataInnerNode>(node));
}

TEST_F(DataTreeGrowingDataTest, GrowAThreeLevelTreeWithLowerLevelFull_DataStaysIntact) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateThreeLevelTreeWithLowerLevelFullWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto node = LoadFirstChildOf(tree->key());
  data.EXPECT_DATA_CORRECT(*dynamic_pointer_move<DataInnerNode>(node));
}

//TODO Test that when growing, the original leaves retains its data with empty and full leaves
//TODO Test tree depth markers on the nodes
//TODO Build-up test cases (build a leaf tree, add N leaves and check end state. End states for example FullTwoLevelTree, FullThreeLevelTree)
