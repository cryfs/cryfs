#include "blockstore/utils/BlockId.h"
#include <cpp-utils/data/Data.h>
#include <cpp-utils/data/DataFixture.h>
#include <gtest/gtest.h>

#include <cstdint>
#include <functional>
#include <map>
#include <set>
#include <vector>

using ::testing::Test;

using blockstore::BlockId;
using cpputils::Data;
using cpputils::DataFixture;

class BlockIdTest : public Test {
public:
    // An id that is all zeroes except for the given byte, which is set to one. Ids built from
    // a larger index are larger, so these can be used to check the ordering.
    static BlockId idWithOneAt(size_t index) {
        Data data(BlockId::BINARY_LENGTH);
        data.FillWithZeroes();
        static_cast<uint8_t*>(data.data())[index] = 1;
        return BlockId::FromBinary(data.data());
    }
};

TEST_F(BlockIdTest, OperatorLess_ComparesById) {
    EXPECT_TRUE(idWithOneAt(1) < idWithOneAt(0));
    EXPECT_FALSE(idWithOneAt(0) < idWithOneAt(1));
}

TEST_F(BlockIdTest, OperatorLess_ComparesTheWholeIdNotJustItsFirstByte) {
    EXPECT_TRUE(idWithOneAt(BlockId::BINARY_LENGTH - 1) < idWithOneAt(BlockId::BINARY_LENGTH - 2));
    EXPECT_FALSE(idWithOneAt(BlockId::BINARY_LENGTH - 2) < idWithOneAt(BlockId::BINARY_LENGTH - 1));
}

TEST_F(BlockIdTest, OperatorLess_NotLessThanItself) {
    const BlockId id(DataFixture::generateFixedSize<BlockId::BINARY_LENGTH>());
    EXPECT_FALSE(id < id);
}

TEST_F(BlockIdTest, OperatorLess_AgreesWithStdLess) {
    const BlockId lhs = idWithOneAt(1);
    const BlockId rhs = idWithOneAt(0);
    EXPECT_EQ(std::less<BlockId>()(lhs, rhs), lhs < rhs);
    EXPECT_EQ(std::less<BlockId>()(rhs, lhs), rhs < lhs);
    EXPECT_EQ(std::less<BlockId>()(lhs, lhs), lhs < lhs);
}

TEST_F(BlockIdTest, CanBeUsedAsMapKey) {
    std::map<BlockId, int> map;
    map.emplace(idWithOneAt(0), 3);
    map.emplace(idWithOneAt(2), 1);
    map.emplace(idWithOneAt(1), 2);
    std::vector<int> valuesInKeyOrder;
    for (const auto& entry : map) {
        valuesInKeyOrder.push_back(entry.second);
    }
    EXPECT_EQ((std::vector<int>{1, 2, 3}), valuesInKeyOrder);
}

TEST_F(BlockIdTest, CanBeUsedAsSetKey) {
    std::set<BlockId> set;
    set.insert(idWithOneAt(0));
    set.insert(idWithOneAt(1));
    set.insert(idWithOneAt(0));
    EXPECT_EQ(2u, set.size());
    EXPECT_TRUE(idWithOneAt(1) == *set.begin());
}
