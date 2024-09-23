#include <cpp-utils/system/get_total_memory.h>
#include <cstdint>
#include <gtest/gtest.h>
#include <stdint.h>

using cpputils::system::get_total_memory;

TEST(GetTotalMemoryTest, DoesntCrash) {
    get_total_memory();
}

TEST(GetTotalMemoryTest, IsNotZero) {
    const uint64_t mem = get_total_memory();
    EXPECT_LT(UINT64_C(0), mem);
}
