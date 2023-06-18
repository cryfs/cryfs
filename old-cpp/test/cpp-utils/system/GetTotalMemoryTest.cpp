#include <gtest/gtest.h>
#include <cpp-utils/system/get_total_memory.h>

using cpputils::system::get_total_memory;

TEST(GetTotalMemoryTest, DoesntCrash) {
    get_total_memory();
}

TEST(GetTotalMemoryTest, IsNotZero) {
    uint64_t mem = get_total_memory();
    EXPECT_LT(UINT64_C(0), mem);
}
