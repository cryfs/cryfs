#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include <gtest/gtest.h>

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <blockstore/implementations/testfake/FakeBlock.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using boost::none;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using std::string;
using ::testing::Test;

using blockstore::BlockId;
using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using cpputils::Data;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

class DataNodeStoreTest : public Test
{
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;

  unique_ref<BlockStore> _blockStore = make_unique_ref<FakeBlockStore>();
  BlockStore *blockStore = _blockStore.get();
  unique_ref<DataNodeStore> nodeStore = make_unique_ref<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES);
};

constexpr uint32_t DataNodeStoreTest::BLOCKSIZE_BYTES;

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type *>(ptr)) << "Given pointer cannot be cast to the given type"

TEST_F(DataNodeStoreTest, PhysicalBlockSize_Leaf)
{
  auto leaf = nodeStore->createNewLeafNode(Data(0));
  auto block = blockStore->load(leaf->blockId()).value();
  EXPECT_EQ(BLOCKSIZE_BYTES, block->size());
}

TEST_F(DataNodeStoreTest, PhysicalBlockSize_Inner)
{
  auto leaf = nodeStore->createNewLeafNode(Data(0));
  auto node = nodeStore->createNewInnerNode(1, {leaf->blockId()});
  auto block = blockStore->load(node->blockId()).value();
  EXPECT_EQ(BLOCKSIZE_BYTES, block->size());
}
