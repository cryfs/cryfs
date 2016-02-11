project (fspp)

set(SOURCES
  impl/FilesystemImpl.cpp
  impl/Profiler.cpp
  fuse/Fuse.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

# This is needed by boost thread
#if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
#  target_link_libraries(${PROJECT_NAME} PRIVATE rt)
#endif(${CMAKE_SYSTEM_NAME} MATCHES "Linux")

target_compile_definitions(${PROJECT_NAME} PUBLIC _FILE_OFFSET_BITS=64)
target_link_libraries(${PROJECT_NAME} PRIVATE cpp-utils)

add_boost(${PROJECT_NAME} filesystem system thread chrono)
enable_style_warnings(${PROJECT_NAME})
activate_cpp14(${PROJECT_NAME})

if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  if(EXISTS "/usr/local/include/osxfuse")
    target_include_directories(${PROJECT_NAME} PRIVATE /usr/local/include/osxfuse)
    target_link_libraries(${PROJECT_NAME} PRIVATE osxfuse)
  else()
    message(FATAL_ERROR "Osxfuse not found in /usr/local/include/osxfuse. Please install osxfuse.")
  endif(EXISTS "/usr/local/include/osxfuse")
else(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  target_link_libraries(${PROJECT_NAME} PRIVATE fuse)
endif(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
