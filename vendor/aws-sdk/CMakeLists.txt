project(aws-sdk)

add_library(${PROJECT_NAME} dummy.cpp)

target_include_directories(${PROJECT_NAME} SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/aws-sdk-cpp)

set(BUILD_ONLY "s3" CACHE BOOL "")
set(BUILD_SHARED_LIBS off CACHE BOOL "")
set(ENABLE_TESTING off CACHE BOOL "")
add_subdirectory(aws-sdk-cpp EXCLUDE_FROM_ALL)

target_link_libraries(${PROJECT_NAME} PUBLIC aws-cpp-sdk-core aws-cpp-sdk-s3)
