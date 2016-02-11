add_subdirectory(gtest-1.7.0)

project (googletest)
add_library(${PROJECT_NAME} dummy.cpp)
target_link_libraries(${PROJECT_NAME} gtest gmock gmock_main)
target_include_directories(${PROJECT_NAME} SYSTEM INTERFACE ${gtest_INCLUDE_DIRS}/include SYSTEM ${gmock_INCLUDE_DIRS}/include)
