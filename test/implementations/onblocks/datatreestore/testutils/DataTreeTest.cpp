#include "DataTreeTest.h"

#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"
#include <messmer/cpp-utils/pointer.h>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using std::make_unique;
using std::unique_ptr;
using std::initializer_list;
using std::vector;
using cpputils::dynamic_pointer_move;

constexpr uint32_t DataTreeTest::BLOCKSIZE_BYTES;

DataTreeTest::DataTreeTest()
  :_nodeStore(make_unique<DataNodeStore>(make_unique<FakeBlockStore>(), BLOCKSIZE_BYTES)),
   nodeStore(_nodeStore.get()),
   treeStore(std::move(_nodeStore)) {
}

unique_ptr<DataLeafNode> DataTreeTest::CreateLeaf() {
  return nodeStore->createNewLeafNode();
}

unique_ptr<DataInnerNode> DataTreeTest::CreateInner(initializer_list<unique_ptr<DataNode>> children) {
  vector<const DataNode*> childrenVector(children.size());
  std::transform(children.begin(), children.end(), childrenVector.begin(), [](const unique_ptr<DataNode> &ptr) {return ptr.get();});
  return CreateInner(childrenVector);
}

unique_ptr<DataInnerNode> DataTreeTest::CreateInner(initializer_list<const DataNode*> children) {
  return CreateInner(vector<const DataNode*>(children));
}

unique_ptr<DataInnerNode> DataTreeTest::CreateInner(vector<const DataNode*> children) {
  assert(children.size() >= 1);
  auto node = nodeStore->createNewInnerNode(**children.begin());
  for(auto child = children.begin()+1; child != children.end(); ++child) {
    node->addChild(**child);
  }
  return node;
}

unique_ptr<DataTree> DataTreeTest::CreateLeafOnlyTree() {
  auto key = CreateLeaf()->key();
  return treeStore.load(key);
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

unique_ptr<DataInnerNode> DataTreeTest::CreateFullTwoLevel() {
  auto root = CreateInner({CreateLeaf().get()});
  FillNode(root.get());
  return root;
}

unique_ptr<DataInnerNode> DataTreeTest::CreateThreeLevelMinData() {
  return CreateInner({
    CreateFullTwoLevel(),
    CreateInner({CreateLeaf()})
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateFourLevelMinData() {
  return CreateInner({
    CreateFullThreeLevel(),
    CreateInner({CreateInner({CreateLeaf()})})
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateFullThreeLevel() {
  auto root = CreateInner({CreateFullTwoLevel().get()});
  FillNodeTwoLevel(root.get());
  return root;
}

unique_ptr<DataInnerNode> DataTreeTest::LoadInnerNode(const Key &key) {
  auto node = nodeStore->load(key);
  auto casted = dynamic_pointer_move<DataInnerNode>(node);
  EXPECT_NE(nullptr, casted.get()) << "Is not an inner node";
  return casted;
}

unique_ptr<DataLeafNode> DataTreeTest::LoadLeafNode(const Key &key) {
  auto node = nodeStore->load(key);
  auto casted =  dynamic_pointer_move<DataLeafNode>(node);
  EXPECT_NE(nullptr, casted.get()) << "Is not a leaf node";
  return casted;
}

unique_ptr<DataInnerNode> DataTreeTest::CreateTwoLeaf() {
  return CreateInner({CreateLeaf().get(), CreateLeaf().get()});
}

unique_ptr<DataTree> DataTreeTest::CreateTwoLeafTree() {
  auto key = CreateTwoLeaf()->key();
  return treeStore.load(key);
}

unique_ptr<DataLeafNode> DataTreeTest::CreateLeafWithSize(uint32_t size) {
  auto leaf = CreateLeaf();
  leaf->resize(size);
  return leaf;
}

unique_ptr<DataInnerNode> DataTreeTest::CreateTwoLeafWithSecondLeafSize(uint32_t size) {
  return CreateInner({
    CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateLeafWithSize(size)
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateFullTwoLevelWithLastLeafSize(uint32_t size) {
  auto root = CreateFullTwoLevel();
  for (uint32_t i = 0; i < root->numChildren()-1; ++i) {
    LoadLeafNode(root->getChild(i)->key())->resize(nodeStore->layout().maxBytesPerLeaf());
  }
  LoadLeafNode(root->LastChild()->key())->resize(size);
  return root;
}

unique_ptr<DataInnerNode> DataTreeTest::CreateThreeLevelWithOneChildAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateThreeLevelWithTwoChildrenAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateThreeLevelWithThreeChildrenAndLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    })
  });
}

unique_ptr<DataInnerNode> DataTreeTest::CreateFullThreeLevelWithLastLeafSize(uint32_t size) {
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

unique_ptr<DataInnerNode> DataTreeTest::CreateFourLevelMinDataWithLastLeafSize(uint32_t size) {
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
