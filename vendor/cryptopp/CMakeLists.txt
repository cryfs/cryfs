project(mycryptopp)

set(BUILD_TESTING OFF)
set(BUILD_DOCUMENTATION OFF)
set(BUILD_SHARED OFF)
set(BUILD_STATIC ON)
add_subdirectory(cryptopp EXCLUDE_FROM_ALL)


add_library(cryptopp dummy.cpp)
target_link_libraries(cryptopp PRIVATE cryptopp-static)
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
