#include <gtest/gtest.h>
#include "blockstore/implementations/ondisk/OnDiskBlockStore2.h"
#include <cpp-utils/tempfile/TempDir.h>
#include <boost/filesystem.hpp>

#include <cstddef>
#include <fstream>

using ::testing::Test;

using cpputils::TempDir;
using cpputils::Data;
using std::ifstream;
using std::ofstream;
using blockstore::BlockId;

using namespace blockstore::ondisk;

class OnDiskBlockStoreTest: public Test {
public:
    OnDiskBlockStoreTest():
    baseDir(),
    blockStore(baseDir.path()) {
  }
  TempDir baseDir;
  OnDiskBlockStore2 blockStore;

  blockstore::BlockId CreateBlockReturnKey(const Data &initData) {
    return blockStore.create(initData.copy());
  }

  uint64_t getPhysicalBlockSize(const BlockId &blockId) {
    ifstream stream((baseDir.path() / blockId.ToString().substr(0,3) / blockId.ToString().substr(3)).c_str());
    stream.seekg(0, stream.end);
    return stream.tellg();
  }

  void writeRawBlockFile(const BlockId &blockId, const void *data, size_t size) {
    std::string const idStr = blockId.ToString();
    auto dir = baseDir.path() / idStr.substr(0, 3);
    boost::filesystem::create_directories(dir);
    auto filepath = dir / idStr.substr(3);
    ofstream file(filepath.string().c_str(), std::ios::binary | std::ios::trunc);
    file.write(static_cast<const char*>(data), static_cast<std::streamsize>(size));
  }
};

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto blockId = CreateBlockReturnKey(Data(0));
  auto baseSize = getPhysicalBlockSize(blockId);
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
  auto blockId = CreateBlockReturnKey(Data(10*1024));
  auto baseSize = getPhysicalBlockSize(blockId);
  EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(baseSize));
}

TEST_F(OnDiskBlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocksWithSameKeyPrefix) {
  const BlockId key1 = BlockId::FromString("4CE72ECDD20877A12ADBF4E3927C0A13");
  const BlockId key2 = BlockId::FromString("4CE72ECDD20877A12ADBF4E3927C0A14");
  EXPECT_TRUE(blockStore.tryCreate(key1, cpputils::Data(0)));
  EXPECT_TRUE(blockStore.tryCreate(key2, cpputils::Data(0)));
  EXPECT_EQ(2u, blockStore.numBlocks());
}

TEST_F(OnDiskBlockStoreTest, LoadingBlockWithEmptyFile_ThrowsError) {
  const BlockId blockId = BlockId::FromString("AB0123456789ABCDEF0123456789AB01");
  writeRawBlockFile(blockId, "", 0);
  EXPECT_THROW(blockStore.load(blockId), std::runtime_error);
}

TEST_F(OnDiskBlockStoreTest, LoadingBlockWithUndersizedFile_ThrowsError) {
  const BlockId blockId = BlockId::FromString("AB0123456789ABCDEF0123456789AB01");
  const char shortData[] = "cryfs";
  writeRawBlockFile(blockId, shortData, sizeof(shortData) - 1);
  EXPECT_THROW(blockStore.load(blockId), std::runtime_error);
}

TEST_F(OnDiskBlockStoreTest, LoadingBlockWithSizeBetweenPrefixAndFullHeader_ThrowsError) {
  // Data larger than FORMAT_VERSION_HEADER_PREFIX but smaller than full header
  const BlockId blockId = BlockId::FromString("AB0123456789ABCDEF0123456789AB01");
  const char partialHeader[] = "cryfs;block;";
  writeRawBlockFile(blockId, partialHeader, sizeof(partialHeader) - 1);
  EXPECT_THROW(blockStore.load(blockId), std::runtime_error);
}
