project (blobstore-test)

set(SOURCES
    implementations/onblocks/utils/MaxZeroSubtractionTest.cpp
    implementations/onblocks/utils/CeilDivisionTest.cpp
    implementations/onblocks/utils/IntPowTest.cpp
    implementations/onblocks/utils/CeilLogTest.cpp
    implementations/onblocks/testutils/BlobStoreTest.cpp
    implementations/onblocks/BlobStoreTest.cpp
    implementations/onblocks/datanodestore/DataLeafNodeTest.cpp
    implementations/onblocks/datanodestore/DataInnerNodeTest.cpp
    implementations/onblocks/datanodestore/DataNodeViewTest.cpp
    implementations/onblocks/datanodestore/DataNodeStoreTest.cpp
    implementations/onblocks/datatreestore/testutils/DataTreeTest.cpp
    implementations/onblocks/datatreestore/impl/GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest.cpp
    implementations/onblocks/datatreestore/impl/GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest.cpp
    implementations/onblocks/datatreestore/DataTreeTest_Performance.cpp
    implementations/onblocks/datatreestore/DataTreeTest_ResizeByTraversing.cpp
    implementations/onblocks/datatreestore/DataTreeTest_NumStoredBytes.cpp
    implementations/onblocks/datatreestore/DataTreeTest_ResizeNumBytes.cpp
    implementations/onblocks/datatreestore/DataTreeStoreTest.cpp
    implementations/onblocks/datatreestore/LeafTraverserTest.cpp
    implementations/onblocks/BlobSizeTest.cpp
    implementations/onblocks/BlobReadWriteTest.cpp
    implementations/onblocks/BigBlobsTest.cpp

)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} my-gtest-main googletest blobstore)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
