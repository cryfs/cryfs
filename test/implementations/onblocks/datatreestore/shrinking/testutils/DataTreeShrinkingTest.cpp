#include <messmer/blobstore/test/implementations/onblocks/datatreestore/shrinking/testutils/DataTreeShrinkingTest.h>

using namespace blobstore::onblocks::datanodestore;

using std::unique_ptr;
using std::make_unique;
using cpputils::dynamic_pointer_move;
using blockstore::Key;
using blobstore::onblocks::datatreestore::DataTree;

void DataTreeShrinkingTest::Shrink(const Key &key) {
  treeStore.load(key)->removeLastDataLeaf();
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateFourNodeThreeLeaf() {
  return CreateInner({CreateLeaf(), CreateLeaf(), CreateLeaf()});
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateTwoInnerNodeOneTwoLeaves() {
  return CreateInner({
    CreateInner({CreateLeaf()}),
    CreateInner({CreateLeaf(), CreateLeaf()})
  });
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateTwoInnerNodeTwoOneLeaves() {
  return CreateInner({
    CreateInner({CreateLeaf(), CreateLeaf()}),
    CreateInner({CreateLeaf()})
  });
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateFourLevelWithTwoSiblingLeaves1() {
  return CreateInner({
    CreateFullThreeLevel(),
    CreateInner({CreateTwoLeaf()})
  });
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateFourLevelWithTwoSiblingLeaves2() {
  return CreateInner({
    CreateFullThreeLevel(),
    CreateInner({CreateFullTwoLevel(), CreateTwoLeaf()})
  });
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel() {
  return CreateInner({
    CreateFullThreeLevel(),
    CreateThreeLevelMinData()
  });
}

unique_ptr<DataInnerNode> DataTreeShrinkingTest::CreateThreeLevelWithThreeChildrenOfRoot() {
  return CreateInner({
    CreateFullTwoLevel(),
    CreateFullTwoLevel(),
    CreateInner({CreateLeaf()})
  });
}
