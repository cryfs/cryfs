#include <gtest/gtest.h>
#include "blockstore/implementations/ondisk/OnDiskBlockStore.h"
#include <cpp-utils/tempfile/TempDir.h>

using ::testing::Test;

using cpputils::TempDir;
using cpputils::Data;
using std::ifstream;

using namespace blockstore::ondisk;

class OnDiskBlockStoreTest: public Test {
public:
    OnDiskBlockStoreTest():
    baseDir(),
    blockStore(baseDir.path()) {
  }
  TempDir baseDir;
  OnDiskBlockStore blockStore;

  blockstore::Key CreateBlockReturnKey(const Data &initData) {
    return blockStore.create(initData)->key();
  }

  uint64_t getPhysicalBlockSize(const blockstore::Key &key) {
    ifstream stream((baseDir.path() / key.ToString()).c_str());
    stream.seekg(0, stream.end);
    return stream.tellg();
  }
};

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto key = CreateBlockReturnKey(Data(0));
  auto baseSize = getPhysicalBlockSize(key);
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(baseSize));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = getPhysicalBlockSize(CreateBlockReturnKey(Data(0)));
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_positive) {
  auto key = CreateBlockReturnKey(Data(10*1024));
  auto baseSize = getPhysicalBlockSize(key);
  EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(baseSize));
}
