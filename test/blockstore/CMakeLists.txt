project (blockstore-test)

set(SOURCES
    utils/BlockStoreUtilsTest.cpp
    implementations/parallelaccess/ParallelAccessBlockStoreTest_Specific.cpp
    implementations/compressing/compressors/testutils/CompressorTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} my-gtest-main googletest blockstore)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
