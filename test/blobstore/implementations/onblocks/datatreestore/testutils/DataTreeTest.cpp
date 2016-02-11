#include "DataTreeTest.h"

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::initializer_list;
using std::vector;
using boost::none;
using cpputils::dynamic_pointer_move;

constexpr uint32_t DataTreeTest::BLOCKSIZE_BYTES;

DataTreeTest::DataTreeTest()
  :_nodeStore(make_unique_ref<DataNodeStore>(make_unique_ref<FakeBlockStore>(), BLOCKSIZE_BYTES)),
   nodeStore(_nodeStore.get()),
   treeStore(std::move(_nodeStore)) {
}

unique_ref<DataLeafNode> DataTreeTest::CreateLeaf() {
  return nodeStore->createNewLeafNode();
}

unique_ref<DataInnerNode> DataTreeTest::CreateInner(initializer_list<unique_ref<DataNode>> children) {
  vector<const DataNode*> childrenVector(children.size());
  std::transform(children.begin(), children.end(), childrenVector.begin(), [](const unique_ref<DataNode> &ptr) {return ptr.get();});
  return CreateInner(childrenVector);
}

unique_ref<DataInnerNode> DataTreeTest::CreateInner(initializer_list<const DataNode*> children) {
  return CreateInner(vector<const DataNode*>(children));
}

unique_ref<DataInnerNode> DataTreeTest::CreateInner(vector<const DataNode*> children) {
  ASSERT(children.size() >= 1, "An inner node must have at least one child");
  auto node = nodeStore->createNewInnerNode(**children.begin());
  for(auto child = children.begin()+1; child != children.end(); ++child) {
    node->addChild(**child);
  }
  return node;
}

unique_ref<DataTree> DataTreeTest::CreateLeafOnlyTree() {
  auto key = CreateLeaf()->key();
  return treeStore.load(key).value();
}

void DataTreeTest::FillNode(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    node->addChild(*CreateLeaf());
  }
}

void DataTreeTest::FillNodeTwoLevel(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    node->addChild(*CreateFullTwoLevel());
  }
}

unique_ref<DataInnerNode> DataTreeTest::CreateFullTwoLevel() {
  auto root = CreateInner({CreateLeaf().get()});
  FillNode(root.get());
  return root;
}

unique_ref<DataInnerNode> DataTreeTest::CreateThreeLevelMinData() {
  return CreateInner({
    CreateFullTwoLevel(),
    CreateInner({CreateLeaf()})
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateFourLevelMinData() {
  return CreateInner({
    CreateFullThreeLevel(),
    CreateInner({CreateInner({CreateLeaf()})})
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateFullThreeLevel() {
  auto root = CreateInner({CreateFullTwoLevel().get()});
  FillNodeTwoLevel(root.get());
  return root;
}

unique_ref<DataInnerNode> DataTreeTest::LoadInnerNode(const Key &key) {
  auto node = nodeStore->load(key).value();
  auto casted = dynamic_pointer_move<DataInnerNode>(node);
  EXPECT_NE(none, casted) << "Is not an inner node";
  return std::move(*casted);
}

unique_ref<DataLeafNode> DataTreeTest::LoadLeafNode(const Key &key) {
  auto node = nodeStore->load(key).value();
  auto casted =  dynamic_pointer_move<DataLeafNode>(node);
  EXPECT_NE(none, casted) << "Is not a leaf node";
  return std::move(*casted);
}

unique_ref<DataInnerNode> DataTreeTest::CreateTwoLeaf() {
  return CreateInner({CreateLeaf().get(), CreateLeaf().get()});
}

unique_ref<DataTree> DataTreeTest::CreateTwoLeafTree() {
  auto key = CreateTwoLeaf()->key();
  return treeStore.load(key).value();
}

unique_ref<DataLeafNode> DataTreeTest::CreateLeafWithSize(uint32_t size) {
  auto leaf = CreateLeaf();
  leaf->resize(size);
  return leaf;
}

unique_ref<DataInnerNode> DataTreeTest::CreateTwoLeafWithSecondLeafSize(uint32_t size) {
  return CreateInner({
    CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateLeafWithSize(size)
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateFullTwoLevelWithLastLeafSize(uint32_t size) {
  auto root = CreateFullTwoLevel();
  for (uint32_t i = 0; i < root->numChildren()-1; ++i) {
    LoadLeafNode(root->getChild(i)->key())->resize(nodeStore->layout().maxBytesPerLeaf());
  }
  LoadLeafNode(root->LastChild()->key())->resize(size);
  return root;
}

unique_ref<DataInnerNode> DataTreeTest::CreateThreeLevelWithOneChildAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateThreeLevelWithTwoChildrenAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateThreeLevelWithThreeChildrenAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ref<DataInnerNode> DataTreeTest::CreateFullThreeLevelWithLastLeafSize(uint32_t size) {
  auto root = CreateFullThreeLevel();
  for (uint32_t i = 0; i < root->numChildren(); ++i) {
    auto node = LoadInnerNode(root->getChild(i)->key());
    for (uint32_t j = 0; j < node->numChildren(); ++j) {
      LoadLeafNode(node->getChild(j)->key())->resize(nodeStore->layout().maxBytesPerLeaf());
    }
  }
  LoadLeafNode(LoadInnerNode(root->LastChild()->key())->LastChild()->key())->resize(size);
  return root;
}

unique_ref<DataInnerNode> DataTreeTest::CreateFourLevelMinDataWithLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullThreeLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({CreateInner({CreateLeafWithSize(size)})})
  });
}

void DataTreeTest::EXPECT_IS_LEAF_NODE(const Key &key) {
  auto node = LoadLeafNode(key);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_INNER_NODE(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_TWONODE_CHAIN(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_EQ(1u, node->numChildren());
  EXPECT_IS_LEAF_NODE(node->getChild(0)->key());
}

void DataTreeTest::EXPECT_IS_FULL_TWOLEVEL_TREE(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), node->numChildren());
  for (unsigned int i = 0; i < node->numChildren(); ++i) {
    EXPECT_IS_LEAF_NODE(node->getChild(i)->key());
  }
}

void DataTreeTest::EXPECT_IS_FULL_THREELEVEL_TREE(const Key &key) {
  auto root = LoadInnerNode(key);
  EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), root->numChildren());
  for (unsigned int i = 0; i < root->numChildren(); ++i) {
    auto node = LoadInnerNode(root->getChild(i)->key());
    EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), node->numChildren());
    for (unsigned int j = 0; j < node->numChildren(); ++j) {
      EXPECT_IS_LEAF_NODE(node->getChild(j)->key());
    }
  }
}

void DataTreeTest::CHECK_DEPTH(int depth, const Key &key) {
  if (depth == 0) {
    EXPECT_IS_LEAF_NODE(key);
  } else {
    auto node = LoadInnerNode(key);
    EXPECT_EQ(depth, node->depth());
    for (uint32_t i = 0; i < node->numChildren(); ++i) {
      CHECK_DEPTH(depth-1, node->getChild(i)->key());
    }
  }
}
