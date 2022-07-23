#include <gtest/gtest.h>
#include <blockstore/implementations/rustbridge/RustBlockStore.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/data/Data.h>
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "blobstore/implementations/onblocks/BlobOnBlocks.h"

using namespace blobstore;
using namespace blobstore::onblocks;
using blockstore::rust::RustBlockStore;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;

// Test cases, ensuring that big blobs (>4G) work (i.e. testing that we don't use any 32bit variables for blob size, etc.)
class BigBlobsTest : public ::testing::Test
{
public:
    static constexpr size_t BLOCKSIZE = 32 * 1024;
    static constexpr uint64_t SMALL_BLOB_SIZE = UINT64_C(1024) * 1024 * 1024 * 3.95; // 3.95 GB (<4GB)
    static constexpr uint64_t LARGE_BLOB_SIZE = UINT64_C(1024) * 1024 * 1024 * 4.05; // 4.05 GB (>4GB)

    static constexpr uint64_t max_uint_32 = std::numeric_limits<uint32_t>::max();
    static_assert(SMALL_BLOB_SIZE < max_uint_32, "LARGE_BLOB_SIZE should need 64bit or the test case is mute");
    static_assert(LARGE_BLOB_SIZE > max_uint_32, "LARGE_BLOB_SIZE should need 64bit or the test case is mute");

    unique_ref<BlobStore> blobStore = make_unique_ref<BlobStoreOnBlocks>(
        make_unique_ref<blockstore::rust::RustBlockStore>(blockstore::rust::bridge::new_locking_compressing_inmemory_blockstore()),
        BLOCKSIZE);
    unique_ref<Blob> blob = blobStore->create();
};

constexpr size_t BigBlobsTest::BLOCKSIZE;
constexpr uint64_t BigBlobsTest::SMALL_BLOB_SIZE;
constexpr uint64_t BigBlobsTest::LARGE_BLOB_SIZE;

TEST_F(BigBlobsTest, Resize)
{
    // These operations are in one test case and not in many small ones, because it takes quite long to create a >4GB blob.

    // Resize to >4GB
    blob->resize(LARGE_BLOB_SIZE);
    EXPECT_EQ(LARGE_BLOB_SIZE, blob->size());

    // Grow while >4GB
    blob->resize(LARGE_BLOB_SIZE + 1024);
    EXPECT_EQ(LARGE_BLOB_SIZE + 1024, blob->size());

    // Shrink while >4GB
    blob->resize(LARGE_BLOB_SIZE);
    EXPECT_EQ(LARGE_BLOB_SIZE, blob->size());

    // Shrink to <4GB
    blob->resize(SMALL_BLOB_SIZE);
    EXPECT_EQ(SMALL_BLOB_SIZE, blob->size());

    // Grow to >4GB
    blob->resize(LARGE_BLOB_SIZE);
    EXPECT_EQ(LARGE_BLOB_SIZE, blob->size());

    // Flush >4GB blob
    blob->flush();

    // Destruct >4GB blob
    auto blockId = blob->blockId();
    cpputils::destruct(std::move(blob));

    // Load >4GB blob
    blob = blobStore->load(blockId).value();

    // Remove >4GB blob
    blobStore->remove(std::move(blob));
}

TEST_F(BigBlobsTest, GrowByWriting_Crossing4GBBorder)
{
    Data fixture = DataFixture::generate(2 * (LARGE_BLOB_SIZE - SMALL_BLOB_SIZE));
    blob->write(fixture.data(), SMALL_BLOB_SIZE, fixture.size());

    EXPECT_EQ(LARGE_BLOB_SIZE + (LARGE_BLOB_SIZE - SMALL_BLOB_SIZE), blob->size());

    Data loaded(fixture.size());
    blob->read(loaded.data(), SMALL_BLOB_SIZE, loaded.size());
    EXPECT_EQ(0, std::memcmp(loaded.data(), fixture.data(), loaded.size()));
}

TEST_F(BigBlobsTest, GrowByWriting_Outside4GBBorder_StartingSizeZero)
{
    Data fixture = DataFixture::generate(1024);
    blob->write(fixture.data(), LARGE_BLOB_SIZE, fixture.size());

    EXPECT_EQ(LARGE_BLOB_SIZE + 1024, blob->size());

    Data loaded(fixture.size());
    blob->read(loaded.data(), LARGE_BLOB_SIZE, loaded.size());
    EXPECT_EQ(0, std::memcmp(loaded.data(), fixture.data(), loaded.size()));
}

TEST_F(BigBlobsTest, GrowByWriting_Outside4GBBorder_StartingSizeOutside4GBBorder)
{
    blob->resize(LARGE_BLOB_SIZE);
    Data fixture = DataFixture::generate(1024);
    blob->write(fixture.data(), LARGE_BLOB_SIZE + 1024, fixture.size());

    EXPECT_EQ(LARGE_BLOB_SIZE + 2048, blob->size());

    Data loaded(fixture.size());
    blob->read(loaded.data(), LARGE_BLOB_SIZE + 1024, loaded.size());
    EXPECT_EQ(0, std::memcmp(loaded.data(), fixture.data(), loaded.size()));
}

TEST_F(BigBlobsTest, ReadWriteAfterGrown_Crossing4GBBorder)
{
    blob->resize(LARGE_BLOB_SIZE + (LARGE_BLOB_SIZE - SMALL_BLOB_SIZE) + 1024);
    Data fixture = DataFixture::generate(2 * (LARGE_BLOB_SIZE - SMALL_BLOB_SIZE));
    blob->write(fixture.data(), SMALL_BLOB_SIZE, fixture.size());

    EXPECT_EQ(LARGE_BLOB_SIZE + (LARGE_BLOB_SIZE - SMALL_BLOB_SIZE) + 1024, blob->size());

    Data loaded(fixture.size());
    blob->read(loaded.data(), SMALL_BLOB_SIZE, loaded.size());
    EXPECT_EQ(0, std::memcmp(loaded.data(), fixture.data(), loaded.size()));
}

TEST_F(BigBlobsTest, ReadWriteAfterGrown_Outside4GBBorder)
{
    blob->resize(LARGE_BLOB_SIZE + 2048);
    Data fixture = DataFixture::generate(1024);
    blob->write(fixture.data(), LARGE_BLOB_SIZE, fixture.size());

    EXPECT_EQ(LARGE_BLOB_SIZE + 2048, blob->size());

    Data loaded(fixture.size());
    blob->read(loaded.data(), LARGE_BLOB_SIZE, loaded.size());
    EXPECT_EQ(0, std::memcmp(loaded.data(), fixture.data(), loaded.size()));
}

// TODO Test Blob::readAll (only on 64bit systems)
