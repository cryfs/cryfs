project (blockstore-test)

set(SOURCES
    utils/BlockStoreUtilsTest.cpp
    interface/BlockStoreTest.cpp
    interface/BlockStore2Test.cpp
    interface/BlockTest.cpp
    implementations/testfake/TestFakeBlockStoreTest.cpp
    implementations/mock/MockBlockStoreTest.cpp
    implementations/inmemory/InMemoryBlockStoreTest.cpp
    implementations/parallelaccess/ParallelAccessBlockStoreTest_Generic.cpp
    implementations/parallelaccess/ParallelAccessBlockStoreTest_Specific.cpp
    implementations/compressing/CompressingBlockStoreTest.cpp
    implementations/compressing/compressors/testutils/CompressorTest.cpp
    implementations/encrypted/EncryptedBlockStoreTest_Generic.cpp
    implementations/encrypted/EncryptedBlockStoreTest_Specific.cpp
    implementations/ondisk/OnDiskBlockStoreTest_Generic.cpp
    implementations/ondisk/OnDiskBlockStoreTest_Specific.cpp
    implementations/ondisk/OnDiskBlockTest/OnDiskBlockCreateTest.cpp
    implementations/ondisk/OnDiskBlockTest/OnDiskBlockFlushTest.cpp
    implementations/ondisk/OnDiskBlockTest/OnDiskBlockLoadTest.cpp
    implementations/caching/CachingBlockStore2Test_Generic.cpp
    implementations/caching/CachingBlockStore2Test_Specific.cpp
    implementations/caching/cache/QueueMapTest_Values.cpp
    implementations/caching/cache/testutils/MinimalKeyType.cpp
    implementations/caching/cache/testutils/CopyableMovableValueType.cpp
    implementations/caching/cache/testutils/MinimalValueType.cpp
    implementations/caching/cache/testutils/QueueMapTest.cpp
    implementations/caching/cache/testutils/CacheTest.cpp
    implementations/caching/cache/QueueMapTest_Size.cpp
    implementations/caching/cache/CacheTest_MoveConstructor.cpp
    implementations/caching/cache/CacheTest_PushAndPop.cpp
    implementations/caching/cache/QueueMapTest_MoveConstructor.cpp
    implementations/caching/cache/QueueMapTest_MemoryLeak.cpp
    implementations/caching/cache/CacheTest_RaceCondition.cpp
    implementations/caching/cache/PeriodicTaskTest.cpp
    implementations/caching/cache/QueueMapTest_Peek.cpp
    implementations/integrity/KnownBlockVersionsTest.cpp
    implementations/integrity/IntegrityBlockStoreTest_Generic.cpp
    implementations/integrity/IntegrityBlockStoreTest_Specific.cpp
    implementations/low2highlevel/LowToHighLevelBlockStoreTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} my-gtest-main blockstore)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
