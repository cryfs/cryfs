#include "blockstore/implementations/compressing/CompressingBlockStore.h"
#include "blockstore/implementations/compressing/compressors/Gzip.h"
#include "blockstore/implementations/compressing/compressors/RunLengthEncoding.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>


using blockstore::BlockStore;
using blockstore::compressing::CompressingBlockStore;
using blockstore::compressing::Gzip;
using blockstore::compressing::RunLengthEncoding;
using blockstore::testfake::FakeBlockStore;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

template<class Compressor>
class CompressingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<CompressingBlockStore<Compressor>>(make_unique_ref<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Compressing_Gzip, BlockStoreTest, CompressingBlockStoreTestFixture<Gzip>);
INSTANTIATE_TYPED_TEST_SUITE_P(Compressing_RunLengthEncoding, BlockStoreTest, CompressingBlockStoreTestFixture<RunLengthEncoding>);
