cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

project(cryfs)

include(utils.cmake)

require_gcc_version(4.8)
require_clang_version(3.7)

# Default value is not to build test cases
if(NOT BUILD_TESTING)
    set(BUILD_TESTING OFF CACHE INTERNAL "BUILD_TESTING")
endif(NOT BUILD_TESTING)

# Default vaule is to build in release mode
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE INTERNAL "CMAKE_BUILD_TYPE")
endif(NOT CMAKE_BUILD_TYPE)

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(cpack)
