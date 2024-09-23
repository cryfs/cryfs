#include <gtest/gtest.h>
#include <cpp-utils/system/memory.h>

using cpputils::UnswappableAllocator;

TEST(MemoryTest, LockingSmallMemoryDoesntCrash) {
  UnswappableAllocator allocator;
  void *data = allocator.allocate(5);
  allocator.free(data, 5);
}

TEST(MemoryTest, LockingLargeMemoryDoesntCrash) {
    UnswappableAllocator allocator;
    void *data = allocator.allocate(10240);
    allocator.free(data, 10240);
}
