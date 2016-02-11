enable_testing()
include_directories(SYSTEM ${gtest_INCLUDE_DIRS}/include SYSTEM ${gmock_INCLUDE_DIRS}/include)
link_libraries(gtest gmock gmock_main)

add_subdirectory(cpp-utils)