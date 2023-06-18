#include <gtest/gtest.h>
#include <cpp-utils/data/SerializationHelper.h>
#include <cpp-utils/data/Data.h>

using cpputils::serialize;
using cpputils::deserialize;
using cpputils::deserializeWithOffset;
using cpputils::Data;

TEST(SerializationHelperTest, uint8) {
    Data data(1);
    serialize<uint8_t>(data.data(), 5u);
    EXPECT_EQ(5u, deserialize<uint8_t>(data.data()));
}

TEST(SerializationHelperTest, int8_positive) {
    Data data(1);
    serialize<int8_t>(data.data(), 5);
    EXPECT_EQ(5, deserialize<int8_t>(data.data()));
}

TEST(SerializationHelperTest, int8_negative) {
    Data data(1);
    serialize<int8_t>(data.data(), -5);
    EXPECT_EQ(-5, deserialize<int8_t>(data.data()));
}

TEST(SerializationHelperTest, uint16_aligned) {
    Data data(2);
    serialize<uint16_t>(data.data(), 1000u);
    EXPECT_EQ(1000u, deserialize<uint16_t>(data.data()));
}

TEST(SerializationHelperTest, uint16_unaligned) {
    Data data(3);
    serialize<uint16_t>(data.dataOffset(1), 1000u);
    EXPECT_EQ(1000u, deserialize<uint16_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int16_postive_aligned) {
    Data data(2);
    serialize<int16_t>(data.data(), 1000);
    EXPECT_EQ(1000, deserialize<int16_t>(data.data()));
}

TEST(SerializationHelperTest, int16_positive_unaligned) {
    Data data(3);
    serialize<int16_t>(data.dataOffset(1), 1000);
    EXPECT_EQ(1000, deserialize<int16_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int16_negative_aligned) {
    Data data(2);
    serialize<int16_t>(data.data(), -1000);
    EXPECT_EQ(-1000, deserialize<int16_t>(data.data()));
}

TEST(SerializationHelperTest, int16_negative_unaligned) {
    Data data(3);
    serialize<int16_t>(data.dataOffset(1), -1000);
    EXPECT_EQ(-1000, deserialize<int16_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, uint32_aligned) {
    Data data(4);
    serialize<uint32_t>(data.data(), 100000u);
    EXPECT_EQ(100000u, deserialize<uint32_t>(data.data()));
}

TEST(SerializationHelperTest, uint32_unaligned) {
    Data data(5);
    serialize<uint32_t>(data.dataOffset(1), 100000u);
    EXPECT_EQ(100000u, deserialize<uint32_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int32_positive_aligned) {
    Data data(4);
    serialize<int32_t>(data.data(), 100000);
    EXPECT_EQ(100000, deserialize<int32_t>(data.data()));
}

TEST(SerializationHelperTest, int32_positive_unaligned) {
    Data data(5);
    serialize<int32_t>(data.dataOffset(1), 100000);
    EXPECT_EQ(100000, deserialize<int32_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int32_negative_aligned) {
    Data data(4);
    serialize<int32_t>(data.data(), -100000);
    EXPECT_EQ(-100000, deserialize<int32_t>(data.data()));
}

TEST(SerializationHelperTest, int32_negative_unaligned) {
    Data data(5);
    serialize<int32_t>(data.dataOffset(1), -100000);
    EXPECT_EQ(-100000, deserialize<int32_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, uint64_aligned) {
    Data data(8);
    serialize<uint64_t>(data.data(), 10000000000u);
    EXPECT_EQ(10000000000u, deserialize<uint64_t>(data.data()));
}

TEST(SerializationHelperTest, uint64_unaligned) {
    Data data(9);
    serialize<uint64_t>(data.dataOffset(1), 10000000000u);
    EXPECT_EQ(10000000000u, deserialize<uint64_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int64_positive_aligned) {
    Data data(8);
    serialize<int64_t>(data.data(), 10000000000);
    EXPECT_EQ(10000000000, deserialize<int64_t>(data.data()));
}

TEST(SerializationHelperTest, int64_positive_unaligned) {
    Data data(9);
    serialize<int64_t>(data.dataOffset(1), 10000000000);
    EXPECT_EQ(10000000000, deserialize<int64_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, int64_negative_aligned) {
    Data data(8);
    serialize<int64_t>(data.data(), -10000000000);
    EXPECT_EQ(-10000000000, deserialize<int64_t>(data.data()));
}

TEST(SerializationHelperTest, int64_negative_unaligned) {
    Data data(9);
    serialize<int64_t>(data.dataOffset(1), -10000000000);
    EXPECT_EQ(-10000000000, deserialize<int64_t>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, float_aligned) {
    Data data(sizeof(float));
    serialize<float>(data.data(), 3.1415f);
    EXPECT_EQ(3.1415f, deserialize<float>(data.data()));
}

TEST(SerializationHelperTest, float_unaligned) {
    Data data(sizeof(float) + 1);
    serialize<float>(data.dataOffset(1), 3.1415f);
    EXPECT_EQ(3.1415f, deserialize<float>(data.dataOffset(1)));
}

TEST(SerializationHelperTest, double_aligned) {
    Data data(sizeof(double));
    serialize<double>(data.data(), 3.1415);
    EXPECT_EQ(3.1415, deserialize<double>(data.data()));
}

TEST(SerializationHelperTest, double_unaligned) {
    Data data(sizeof(double) + 1);
    serialize<double>(data.dataOffset(1), 3.1415);
    EXPECT_EQ(3.1415, deserialize<double>(data.dataOffset(1)));
}

namespace {
struct DataStructure final {
    uint64_t v1;
    uint32_t v2;
    uint16_t v3;
    uint8_t v4;
};

bool operator==(const DataStructure &lhs, const DataStructure &rhs) {
    return lhs.v1 == rhs.v1 && lhs.v2 == rhs.v2 && lhs.v3 == rhs.v3 && lhs.v4 == rhs.v4;
}
}

TEST(SerializationHelperTest, struct_aligned) {
    Data data(sizeof(DataStructure));
    const DataStructure fixture {10000000000u, 100000u, 1000u, 5u};
    serialize<DataStructure>(data.data(), fixture);
    EXPECT_EQ(fixture, deserialize<DataStructure>(data.data()));
}

TEST(SerializationHelperTest, struct_unaligned) {
    Data data(sizeof(DataStructure) + 1);
    const DataStructure fixture {10000000000u, 100000u, 1000u, 5u};
    serialize<DataStructure>(data.dataOffset(1), fixture);
    EXPECT_EQ(fixture, deserialize<DataStructure>(data.dataOffset(1)));
}

namespace {
struct OneByteStruct final {
    uint8_t v;
};
static_assert(sizeof(OneByteStruct) == 1, "");
}

TEST(SerializationHelperTest, onebytestruct) {
    Data data(1);
    OneByteStruct fixture {5};
    serialize<OneByteStruct>(data.data(), fixture);
    EXPECT_EQ(fixture.v, deserialize<OneByteStruct>(data.data()).v);
}

TEST(SerializationHelperTest, deserializeWithOffset) {
    Data data(5);
    serialize<uint16_t>(data.dataOffset(1), 1000);
    EXPECT_EQ(1000, deserializeWithOffset<uint16_t>(data.data(), 1));
}
