project (blockstore)

set(SOURCES
  utils/Key.cpp
  utils/BlockStoreUtils.cpp
  utils/FileDoesntExistException.cpp
  interface/helpers/BlockStoreWithRandomKeys.cpp
  implementations/testfake/FakeBlockStore.cpp
  implementations/testfake/FakeBlock.cpp
  implementations/inmemory/InMemoryBlock.cpp
  implementations/inmemory/InMemoryBlockStore.cpp
  implementations/parallelaccess/ParallelAccessBlockStore.cpp
  implementations/parallelaccess/BlockRef.cpp
  implementations/parallelaccess/ParallelAccessBlockStoreAdapter.cpp
  implementations/compressing/CompressingBlockStore.cpp
  implementations/compressing/CompressedBlock.cpp
  implementations/compressing/compressors/RunLengthEncoding.cpp
  implementations/compressing/compressors/Gzip.cpp
  implementations/encrypted/EncryptedBlockStore.cpp
  implementations/encrypted/EncryptedBlock.cpp
  implementations/ondisk/OnDiskBlockStore.cpp
  implementations/ondisk/OnDiskBlock.cpp
  implementations/caching/CachingBlockStore.cpp
  implementations/caching/cache/PeriodicTask.cpp
  implementations/caching/cache/CacheEntry.cpp
  implementations/caching/cache/Cache.cpp
  implementations/caching/cache/QueueMap.cpp
  implementations/caching/CachedBlock.cpp
  implementations/caching/NewBlock.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils)

target_add_boost(${PROJECT_NAME} filesystem system thread)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
