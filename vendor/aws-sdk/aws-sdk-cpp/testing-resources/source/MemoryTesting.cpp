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


#include <aws/testing/MemoryTesting.h>

#include <aws/external/gtest.h>

#include <aws/core/utils/UnreferencedParam.h>

#include <chrono>
#include <thread>
#include <cstdlib>
#include <cstddef>
#if defined(_MSC_VER) && _MSC_VER < 1900
#define alignof __alignof
#endif


namespace {
#if defined(__GNUC__) && __GNUC__ == 4 && __GNUC_MINOR__ <= 8 && !defined(__clang__)
    // GCC 4.8 has `max_align_t` defined in global namespace
    using ::max_align_t;
#else
    using std::max_align_t;
#endif
}


BaseTestMemorySystem::BaseTestMemorySystem() :
    m_currentBytesAllocated(0),
    m_maxBytesAllocated(0),
    m_totalBytesAllocated(0),
    m_currentOutstandingAllocations(0),
    m_maxOutstandingAllocations(0),
    m_totalAllocations(0)
{
}

void* BaseTestMemorySystem::AllocateMemory(std::size_t blockSize, std::size_t alignment, const char *allocationTag) 
{
    AWS_UNREFERENCED_PARAM(alignment);
    AWS_UNREFERENCED_PARAM(allocationTag);

    ++m_currentOutstandingAllocations;
    m_maxOutstandingAllocations = (std::max)(m_maxOutstandingAllocations, m_currentOutstandingAllocations);
    ++m_totalAllocations;
            
    m_currentBytesAllocated += blockSize;
    m_maxBytesAllocated = (std::max)(m_maxBytesAllocated, m_currentBytesAllocated);
    m_totalBytesAllocated += blockSize;

    // Note: malloc will always return an address aligned with alignof(std::max_align_t);
    // This alignment value is not always equals to sizeof(std::size_t). But one thing we can make sure is that
    // alignof(std::max_align_t) is always multiple of sizeof(std::size_t).
    // On some platforms, in place construction requires memory address must be aligned with alignof(std::max_align_t).
    // E.g on Mac, x86_64, sizeof(std::size_t) equals 8. but alignof(std::max_align_t) equals 16. std::function requires aligned memory address.
    // To record the malloc size and keep returned address align with 16, instead of malloc extra 8 bytes,  
    // we end up with malloc extra 16 bytes.
    
    char* rawMemory = reinterpret_cast<char*>(malloc(blockSize + alignof(max_align_t)));
    std::size_t *pointerToSize = reinterpret_cast<std::size_t*>(reinterpret_cast<void*>(rawMemory));
    *pointerToSize = blockSize;

    return reinterpret_cast<void*>(rawMemory + alignof(max_align_t));
}

void BaseTestMemorySystem::FreeMemory(void* memoryPtr) 
{
    ASSERT_NE(m_currentOutstandingAllocations, 0ULL);
    if(m_currentOutstandingAllocations > 0)
    {
        --m_currentOutstandingAllocations;
    }

    std::size_t *pointerToSize = reinterpret_cast<std::size_t*>(reinterpret_cast<char*>(memoryPtr) - alignof(max_align_t));
    std::size_t blockSize = *pointerToSize;

    ASSERT_GE(m_currentBytesAllocated, blockSize);
    if(m_currentBytesAllocated >= blockSize)
    {
        m_currentBytesAllocated -= blockSize;
    }

    free(reinterpret_cast<void*>(pointerToSize));
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

ExactTestMemorySystem::ExactTestMemorySystem(uint32_t bucketCount, uint32_t trackersPerBlock) :
    Base(),
    m_bucketCount(bucketCount),
    m_trackersPerBlock(trackersPerBlock),
    m_blocks(nullptr),
    m_freePool(nullptr),
    m_buckets(nullptr),
    m_internalSync()
{
    m_buckets = reinterpret_cast<TaggedMemoryTracker**>(malloc(bucketCount * sizeof(TaggedMemoryTracker*)));

    for(uint32_t i = 0; i < bucketCount; ++i)
    {
        m_buckets[i] = nullptr;
    }
}

ExactTestMemorySystem::~ExactTestMemorySystem()
{
    Cleanup();
}

void ExactTestMemorySystem::Cleanup()
{
    // free all elements in the m_blocks linked list
    while(m_blocks != nullptr)
    {
        RawBlock* block = m_blocks;
        m_blocks = m_blocks->m_next;

        free(block);
    }

    free(m_buckets);
}

void ExactTestMemorySystem::GrowFreePool()
{
    // malloc enough memory to hold the linked list pointer as well as the desired number of TaggedMemoryTrackers
    RawBlock* block = reinterpret_cast<RawBlock*>(malloc(m_trackersPerBlock * sizeof(TaggedMemoryTracker) + sizeof(RawBlock*)));
    block->m_next = m_blocks;
    m_blocks = block;

    // for each embedded TaggedMemoryTracker, initialize it and push it onto the free pool list
    TaggedMemoryTracker* tracker = reinterpret_cast<TaggedMemoryTracker*>(reinterpret_cast<char *>(block) + sizeof(RawBlock*));
    for(uint32_t i = 0; i < m_trackersPerBlock; ++i)
    {
        tracker->m_next = m_freePool;
        tracker->m_tag = nullptr;
        tracker->m_memory = nullptr;
        tracker->m_size = 0;

        m_freePool = tracker;
        ++tracker;
    }
}

// takes a memory address and returns a hash bucket index
uint32_t ExactTestMemorySystem::CalculateBucketIndex(const void* memory) const
{
    uint64_t address = reinterpret_cast<uint64_t>(memory);
    address /= (sizeof(void *));  // it's likely that the returns from malloc are aligned via pointer-size, so let's divide that out to get better distribution

    return address % m_bucketCount;
}

ExactTestMemorySystem::TaggedMemoryTracker* ExactTestMemorySystem::AllocateTracker()
{
    if(m_freePool == nullptr)
    {
        GrowFreePool();
    }

    TaggedMemoryTracker* tracker = m_freePool;
    m_freePool = m_freePool->m_next;
    return tracker;
}

void* ExactTestMemorySystem::AllocateMemory(std::size_t blockSize, std::size_t alignment, const char *allocationTag) 
{
    std::lock_guard<std::mutex> lock(m_internalSync);

    void* rawMemory = Base::AllocateMemory(blockSize, alignment, allocationTag);

    uint32_t bucketIndex = CalculateBucketIndex(rawMemory);

    TaggedMemoryTracker* tracker = AllocateTracker();
    tracker->m_next = m_buckets[bucketIndex];
    tracker->m_memory = rawMemory;
    tracker->m_tag = allocationTag;
    tracker->m_size = blockSize;
    
    m_buckets[bucketIndex] = tracker;

    return rawMemory;
}

void ExactTestMemorySystem::FreeMemory(void* memoryPtr) 
{
    std::lock_guard<std::mutex> lock(m_internalSync);

    uint32_t bucketIndex = CalculateBucketIndex(memoryPtr);
    bool foundMemory = false;
    TaggedMemoryTracker** prevPtr = &m_buckets[bucketIndex];
    TaggedMemoryTracker* currentTracker = m_buckets[bucketIndex];
    while(currentTracker != nullptr)
    {
        if(currentTracker->m_memory == memoryPtr)
        {
            // we found its TaggedMemoryTracker, splice it out of the list and return it to the free pool
            *prevPtr = currentTracker->m_next;
            currentTracker->m_next = m_freePool;
            m_freePool = currentTracker;
            foundMemory = true;
            break;
        }

        prevPtr = &(currentTracker->m_next);
        currentTracker = currentTracker->m_next;
    }

    if(!foundMemory)
    {
        // we have no record of this, that's bad, let's not free it
        return;
    }

    Base::FreeMemory(memoryPtr);
}

// true iff all allocations have been freed
bool ExactTestMemorySystem::IsClean() const
{
    for(uint32_t i = 0; i < m_bucketCount; ++i)
    {
        if(m_buckets[i] != nullptr)
        {
            return false;
        }
    }

    return true;
}

