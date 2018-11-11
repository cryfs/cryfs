/*
 * Copyright 2010-2017 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License").
 * You may not use this file except in compliance with the License.
 * A copy of the License is located at
 * 
 *  http://aws.amazon.com/apache2.0
 * 
 * or in the "license" file accompanying this file. This file is distributed
 * on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing
 * permissions and limitations under the License.
 */

#pragma once

#include <aws/testing/Testing_EXPORTS.h>

#include <aws/core/utils/memory/MemorySystemInterface.h>
#include <aws/core/utils/memory/AWSMemory.h>

#include <stdint.h>
#include <algorithm>
#include <mutex>
#include <atomic>
#include <cstdlib>

// Could be folded into ExactTestMemorySystem, tracks some aggregate stats
class AWS_TESTING_API BaseTestMemorySystem : public Aws::Utils::Memory::MemorySystemInterface
{
    public:

        BaseTestMemorySystem();
        virtual ~BaseTestMemorySystem() {}

        virtual void Begin() override{}
        virtual void End() override {}

        virtual void* AllocateMemory(std::size_t blockSize, std::size_t alignment, const char *allocationTag = nullptr) override;
        virtual void FreeMemory(void* memoryPtr) override;

        uint64_t GetCurrentOutstandingAllocations() const { return m_currentOutstandingAllocations; }
        uint64_t GetMaxOutstandingAllocations() const { return m_maxOutstandingAllocations; }
        uint64_t GetTotalAllocationCount() const { return m_totalAllocations; }

        uint64_t GetCurrentBytesAllocated() const { return m_currentBytesAllocated; }
        uint64_t GetMaxBytesAllocated() const { return m_maxBytesAllocated; }
        uint64_t GetTotalBytesAllocated() const { return m_totalBytesAllocated; }

    private:

        uint64_t m_currentBytesAllocated;
        uint64_t m_maxBytesAllocated;
        uint64_t m_totalBytesAllocated;

        uint64_t m_currentOutstandingAllocations;
        uint64_t m_maxOutstandingAllocations;
        uint64_t m_totalAllocations;
};

// This is thread-safe; while active it keeps a record of every single allocation made via the memory system allowing us to verify matching deallocations
class AWS_TESTING_API ExactTestMemorySystem : public BaseTestMemorySystem
{
    public:

        typedef BaseTestMemorySystem Base;

        ExactTestMemorySystem(uint32_t bucketCount, uint32_t trackersPerBlock);
        virtual ~ExactTestMemorySystem();

        virtual void* AllocateMemory(std::size_t blockSize, std::size_t alignment, const char *allocationTag = nullptr) override;
        virtual void FreeMemory(void* memoryPtr) override;

        bool IsClean() const;

    private:

        // C-style memory tracking

        // This is the element of internal allocation, containing one or more TaggedMemoryTrackers (# based on m_trackersPerBlock)
        // This allows us to scale the tracker to tests that do a lot of allocation without having one malloc per TaggedMemoryTracker
        // POD
        struct RawBlock
        {
            RawBlock* m_next;
        };

        // TaggedMemoryTrackers are ultimately just offsets inside RawBlocks
        // POD
        struct TaggedMemoryTracker
        {
            TaggedMemoryTracker* m_next;
            size_t m_size;
            const char* m_tag;
            const void* m_memory;
        };

        uint32_t CalculateBucketIndex(const void* memory) const;
        TaggedMemoryTracker* AllocateTracker();
        void GrowFreePool();
        void Cleanup();

        uint32_t m_bucketCount;
        uint32_t m_trackersPerBlock;

        // A linked list of all malloc'd RawBlocks; this is what we free during cleanup
        RawBlock* m_blocks;

        // A linked list of available TaggedMemoryTrackers
        TaggedMemoryTracker* m_freePool;

        // An array of linked lists of TaggedMemoryTrackers; tracks all allocations made while this system is active; the array size is controlled by m_bucketCount
        TaggedMemoryTracker** m_buckets;

        // Keeps allocation/deallocation synchronous so that all of our bookkeeping actually works properly
        std::mutex m_internalSync;

};

#ifdef USE_AWS_MEMORY_MANAGEMENT

// Utility macros to put at the start and end of tests
// Checks:
//   (1) Everything allocated by the AWS memory system is deallocated

// Macros that can be used to bracket the inside of a gtest body
#define AWS_BEGIN_MEMORY_TEST(x, y)   ExactTestMemorySystem memorySystem(x, y); \
                                      Aws::Utils::Memory::InitializeAWSMemorySystem(memorySystem); \
                                      {  

#define AWS_END_MEMORY_TEST           } \
                                      Aws::Utils::Memory::ShutdownAWSMemorySystem(); \
                                      ASSERT_EQ(memorySystem.GetCurrentOutstandingAllocations(), 0ULL); \
                                      ASSERT_EQ(memorySystem.GetCurrentBytesAllocated(), 0ULL); \
                                      ASSERT_TRUE(memorySystem.IsClean()); 

#define AWS_END_MEMORY_OVERRIDE   } \
                                  Aws::Utils::Memory::ShutdownAWSMemorySystem();

#define AWS_BEGIN_MEMORY_TEST_EX(options, x, y) ExactTestMemorySystem memorySystem(x, y); \
                                                options.memoryManagementOptions.memoryManager = &memorySystem;

#define AWS_END_MEMORY_TEST_EX                  EXPECT_EQ(memorySystem.GetCurrentOutstandingAllocations(), 0ULL); \
                                                EXPECT_EQ(memorySystem.GetCurrentBytesAllocated(), 0ULL); \
                                                EXPECT_TRUE(memorySystem.IsClean()); 
#else

#define AWS_BEGIN_MEMORY_TEST(x, y)
#define AWS_END_MEMORY_TEST
#define AWS_END_MEMORY_OVERRIDE
#define AWS_BEGIN_MEMORY_TEST_EX(options, x, y)
#define AWS_END_MEMORY_TEST_EX

#endif // USE_AWS_MEMORY_MANAGEMENT
