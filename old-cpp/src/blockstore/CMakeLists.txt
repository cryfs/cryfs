project (blockstore)

set(SOURCES
  utils/BlockId.cpp
  utils/IdWrapper.cpp
  utils/BlockStoreUtils.cpp
  utils/FileDoesntExistException.cpp
  implementations/testfake/FakeBlockStore.cpp
  implementations/testfake/FakeBlock.cpp
  implementations/parallelaccess/ParallelAccessBlockStore.cpp
  implementations/parallelaccess/BlockRef.cpp
  implementations/parallelaccess/ParallelAccessBlockStoreAdapter.cpp
  implementations/caching/cache/PeriodicTask.cpp
  implementations/caching/cache/CacheEntry.cpp
  implementations/caching/cache/Cache.cpp
  implementations/caching/cache/QueueMap.cpp
  implementations/mock/MockBlockStore.cpp
  implementations/mock/MockBlock.cpp
  implementations/rustbridge/RustBlockStore2.cpp
  implementations/rustbridge/RustBlockStore.cpp
  implementations/rustbridge/RustBlock.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils)

target_add_boost(${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

include (rust)
target_add_rust_companion(
    blockstore
    RUST_LIB_NAME "libcryfs_cppbridge.a"
    RUST_CRATE_NAME "cryfs-cppbridge"
    RUST_BRIDGES "src/blockstore.rs" "src/blobstore.rs" "src/fsblobstore.rs"
    RUST_DIR "rust"
)
