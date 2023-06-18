#include "DataTreeTest.h"

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::mock::MockBlockStore;
using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::initializer_list;
using std::vector;
using boost::none;
using cpputils::dynamic_pointer_move;
using cpputils::Data;

constexpr uint32_t DataTreeTest::BLOCKSIZE_BYTES;

DataTreeTest::DataTreeTest()
  :_blockStore(make_unique_ref<MockBlockStore>()),
   blockStore(_blockStore.get()),
   _nodeStore(make_unique_ref<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES)),
   nodeStore(_nodeStore.get()),
   treeStore(std::move(_nodeStore)) {
}

unique_ref<DataLeafNode> DataTreeTest::CreateLeaf() {
  return nodeStore->createNewLeafNode(Data(nodeStore->layout().maxBytesPerLeaf()));
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
  vector<BlockId> childrenKeys;
  childrenKeys.reserve(children.size());
  for (const DataNode *child : children) {
    ASSERT(child->depth() == (*children.begin())->depth(), "Children with different depth");
    childrenKeys.push_back(child->blockId());
  }
  auto node = nodeStore->createNewInnerNode((*children.begin())->depth()+1, childrenKeys);
  return node;
}

unique_ref<DataTree> DataTreeTest::CreateLeafOnlyTree() {
  auto blockId = CreateLeaf()->blockId();
  return treeStore.load(blockId).value();
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

unique_ref<DataInnerNode> DataTreeTest::LoadInnerNode(const BlockId &blockId) {
  auto node = nodeStore->load(blockId).value();
  auto casted = dynamic_pointer_move<DataInnerNode>(node);
  EXPECT_NE(none, casted) << "Is not an inner node";
  return std::move(*casted);
}

unique_ref<DataLeafNode> DataTreeTest::LoadLeafNode(const BlockId &blockId) {
  auto node = nodeStore->load(blockId).value();
  auto casted =  dynamic_pointer_move<DataLeafNode>(node);
  EXPECT_NE(none, casted) << "Is not a leaf node";
  return std::move(*casted);
}

unique_ref<DataInnerNode> DataTreeTest::CreateTwoLeaf() {
  return CreateInner({CreateLeaf().get(), CreateLeaf().get()});
}

unique_ref<DataTree> DataTreeTest::CreateTwoLeafTree() {
  auto blockId = CreateTwoLeaf()->blockId();
  return treeStore.load(blockId).value();
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
    LoadLeafNode(root->readChild(i).blockId())->resize(nodeStore->layout().maxBytesPerLeaf());
  }
  LoadLeafNode(root->readLastChild().blockId())->resize(size);
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
    auto node = LoadInnerNode(root->readChild(i).blockId());
    for (uint32_t j = 0; j < node->numChildren(); ++j) {
      LoadLeafNode(node->readChild(j).blockId())->resize(nodeStore->layout().maxBytesPerLeaf());
    }
  }
  LoadLeafNode(LoadInnerNode(root->readLastChild().blockId())->readLastChild().blockId())->resize(size);
  return root;
}

unique_ref<DataInnerNode> DataTreeTest::CreateFourLevelMinDataWithLastLeafSize(uint32_t size) {
  return CreateInner({
    CreateFullThreeLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
    CreateInner({CreateInner({CreateLeafWithSize(size)})})
  });
}

void DataTreeTest::EXPECT_IS_LEAF_NODE(const BlockId &blockId) {
  auto node = LoadLeafNode(blockId);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_INNER_NODE(const BlockId &blockId) {
  auto node = LoadInnerNode(blockId);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_TWONODE_CHAIN(const BlockId &blockId) {
  auto node = LoadInnerNode(blockId);
  EXPECT_EQ(1u, node->numChildren());
  EXPECT_IS_LEAF_NODE(node->readChild(0).blockId());
}

void DataTreeTest::EXPECT_IS_FULL_TWOLEVEL_TREE(const BlockId &blockId) {
  auto node = LoadInnerNode(blockId);
  EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), node->numChildren());
  for (unsigned int i = 0; i < node->numChildren(); ++i) {
    EXPECT_IS_LEAF_NODE(node->readChild(i).blockId());
  }
}

void DataTreeTest::EXPECT_IS_FULL_THREELEVEL_TREE(const BlockId &blockId) {
  auto root = LoadInnerNode(blockId);
  EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), root->numChildren());
  for (unsigned int i = 0; i < root->numChildren(); ++i) {
    auto node = LoadInnerNode(root->readChild(i).blockId());
    EXPECT_EQ(nodeStore->layout().maxChildrenPerInnerNode(), node->numChildren());
    for (unsigned int j = 0; j < node->numChildren(); ++j) {
      EXPECT_IS_LEAF_NODE(node->readChild(j).blockId());
    }
  }
}

// NOLINTNEXTLINE(misc-no-recursion)
void DataTreeTest::CHECK_DEPTH(int depth, const BlockId &blockId) {
  if (depth == 0) {
    EXPECT_IS_LEAF_NODE(blockId);
  } else {
    auto node = LoadInnerNode(blockId);
    EXPECT_EQ(depth, node->depth());
    for (uint32_t i = 0; i < node->numChildren(); ++i) {
      CHECK_DEPTH(depth-1, node->readChild(i).blockId());
    }
  }
}
