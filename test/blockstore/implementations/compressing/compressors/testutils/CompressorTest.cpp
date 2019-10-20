#include <gtest/gtest.h>
#include "blockstore/implementations/compressing/compressors/Gzip.h"
#include "blockstore/implementations/compressing/compressors/RunLengthEncoding.h"
#include <cpp-utils/data/DataFixture.h>

using namespace blockstore::compressing;
using cpputils::Data;
using cpputils::DataFixture;

template<class Compressor>
class CompressorTest: public ::testing::Test {
public:
    void EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(const Data &data) {
        Data compressed = Compressor::Compress(data);
        Data decompressed = Compressor::Decompress(compressed.data(), compressed.size());
        EXPECT_EQ(data, decompressed);
    }
};

TYPED_TEST_SUITE_P(CompressorTest);

TYPED_TEST_P(CompressorTest, Empty) {
  Data empty(0);
  this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(empty);
}

TYPED_TEST_P(CompressorTest, ArbitraryData) {
  Data fixture = DataFixture::generate(10240);
  this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(fixture);
}

TYPED_TEST_P(CompressorTest, Zeroes) {
  Data zeroes(10240);
  zeroes.FillWithZeroes();
  this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(zeroes);
}

TYPED_TEST_P(CompressorTest, Runs) {
    Data data(4096);
    std::memset(data.dataOffset(0),    0xF2, 1024);
    std::memset(data.dataOffset(1024), 0x00, 1024);
    std::memset(data.dataOffset(2048), 0x01, 2048);
    this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(data);
}

TYPED_TEST_P(CompressorTest, RunsAndArbitrary) {
    Data data(4096);
    std::memset(data.dataOffset(0),    0xF2, 1024);
    std::memcpy(data.dataOffset(1024), DataFixture::generate(1024).data(), 1024);
    std::memset(data.dataOffset(2048), 0x01, 2048);
    this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(data);
}

TYPED_TEST_P(CompressorTest, LargeData) {
    // this is larger than what fits into 16bit (16bit are for example used as run length indicator in RunLengthEncoding)
    Data fixture = DataFixture::generate(200000);
    this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(fixture);
}

TYPED_TEST_P(CompressorTest, LargeRuns) {
    // each run is larger than what fits into 16bit (16bit are for example used as run length indicator in RunLengthEncoding)
    constexpr size_t RUN_SIZE = 200000;
    Data data(3*RUN_SIZE);
    std::memset(data.dataOffset(0),          0xF2, RUN_SIZE);
    std::memset(data.dataOffset(RUN_SIZE),   0x00, RUN_SIZE);
    std::memset(data.dataOffset(2*RUN_SIZE), 0x01, RUN_SIZE);
    this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(data);
}

TYPED_TEST_P(CompressorTest, LargeRunsAndArbitrary) {
    // each run is larger than what fits into 16bit (16bit are for example used as run length indicator in RunLengthEncoding)
    constexpr size_t RUN_SIZE = 200000;
    Data data(3*RUN_SIZE);
    std::memset(data.dataOffset(0), 0xF2, RUN_SIZE);
    std::memcpy(data.dataOffset(RUN_SIZE), DataFixture::generate(RUN_SIZE).data(), RUN_SIZE);
    std::memset(data.dataOffset(2*RUN_SIZE), 0x01, RUN_SIZE);
    this->EXPECT_COMPRESS_AND_DECOMPRESS_IS_IDENTITY(data);
}

REGISTER_TYPED_TEST_SUITE_P(CompressorTest,
        Empty,
        ArbitraryData,
        Zeroes,
        Runs,
        RunsAndArbitrary,
        LargeData,
        LargeRuns,
        LargeRunsAndArbitrary
);

INSTANTIATE_TYPED_TEST_SUITE_P(Gzip, CompressorTest, Gzip);
INSTANTIATE_TYPED_TEST_SUITE_P(RunLengthEncoding, CompressorTest, RunLengthEncoding);
