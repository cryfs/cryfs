#include <cpp-utils/data/DataFixture.h>
#include "blockstore/utils/FileDoesntExistException.h"
#include <gtest/gtest.h>

#include <cpp-utils/data/Data.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <fstream>

// TODO This should be ported to BlockStore2
/*
using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::ofstream;
using std::ios;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::TempFile;
using cpputils::TempDir;
using cpputils::unique_ref;

using namespace blockstore;
using namespace blockstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlockLoadTest: public Test, public WithParamInterface<size_t> {
public:
  OnDiskBlockLoadTest():
    dir(),
    key(BlockId::FromString("1491BB4932A389EE14BC7090AC772972")),
    file(dir.path() / blockId.ToString(), false) {
  }
  TempDir dir;
  BlockId key;
  TempFile file;

  void CreateBlockWithSize(size_t size) {
    Data data(size);
    OnDiskBlock::CreateOnDisk(dir.path(), blockId, std::move(data));
  }

  void StoreData(Data data) {
    OnDiskBlock::CreateOnDisk(dir.path(), blockId, std::move(data));
  }

  unique_ref<OnDiskBlock> LoadBlock() {
    return OnDiskBlock::LoadFromDisk(dir.path(), blockId).value();
  }

  void EXPECT_BLOCK_DATA_EQ(const Data &expected, const OnDiskBlock &actual) {
    EXPECT_EQ(expected.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));
  }
};
INSTANTIATE_TEST_SUITE_P(OnDiskBlockLoadTest, OnDiskBlockLoadTest, Values(0, 1, 5, 1024, 10*1024*1024));

TEST_P(OnDiskBlockLoadTest, LoadsCorrectSize) {
  CreateBlockWithSize(GetParam());

  auto block = LoadBlock();

  EXPECT_EQ(GetParam(), block->size());
}

TEST_P(OnDiskBlockLoadTest, LoadedDataIsCorrect) {
  Data randomData = DataFixture::generate(GetParam());
  StoreData(randomData.copy());

  auto block = LoadBlock();

  EXPECT_BLOCK_DATA_EQ(randomData, *block);
}

TEST_F(OnDiskBlockLoadTest, LoadNotExistingBlock) {
  BlockId key2 = BlockId::FromString("272EE5517627CFA147A971A8E6E747E0");
  EXPECT_EQ(boost::none, OnDiskBlock::LoadFromDisk(dir.path(), key2));
}
*/
