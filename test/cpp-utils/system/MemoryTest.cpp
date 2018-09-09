#include <gtest/gtest.h>
#include <cpp-utils/system/memory.h>
#include <memory>
#include <cpp-utils/pointer/gcc_4_8_compatibility.h>

using cpputils::DontSwapMemoryRAII;

TEST(MemoryTest, LockingSmallStackMemoryDoesntCrash) {
  bool memory;
  DontSwapMemoryRAII obj(&memory, sizeof(memory));
}

TEST(MemoryTest, LockingLargeStackMemoryDoesntCrash) {
    bool memory[10*1024];
    DontSwapMemoryRAII obj(memory, sizeof(memory));
}

TEST(MemoryTest, LockingSmallHeapMemoryDoesntCrash) {
    auto memory = std::make_unique<bool>(false);
    DontSwapMemoryRAII obj(memory.get(), sizeof(*memory));
}

TEST(MemoryTest, LockingLargeHeapMemoryDoesntCrash) {
    auto memory = std::make_unique<bool[]>(10*1024);
    DontSwapMemoryRAII obj(memory.get(), 10*1024);
}
