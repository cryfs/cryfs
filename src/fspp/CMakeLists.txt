project (fspp)

set(SOURCES
  impl/FilesystemImpl.cpp
  impl/Profiler.cpp
  fuse/Fuse.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_compile_definitions(${PROJECT_NAME} PUBLIC _FILE_OFFSET_BITS=64)
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils)

target_add_boost(${PROJECT_NAME} filesystem system thread chrono)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(CMAKE_FIND_FRAMEWORK LAST)
  set(FUSE_LIB_NAME "osxfuse")
else(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(FUSE_LIB_NAME "fuse")
endif(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")

find_library_with_path(FUSE ${FUSE_LIB_NAME} FUSE_LIB_PATH)
target_link_libraries(${PROJECT_NAME} PUBLIC ${FUSE})
