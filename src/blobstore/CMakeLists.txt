project (blobstore)

set(SOURCES
    implementations/onblocks/utils/Math.cpp
    implementations/rustbridge/RustBlob.cpp
    implementations/rustbridge/RustBlobStore.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils blockstore)

target_add_boost(${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
