cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

# note: for clang-tidy, we need cmake 3.6, or (if the return code should be handled correctly, e.g. on CI), we need 3.8.

project(cryfs)

include(utils.cmake)

require_gcc_version(4.9)
require_clang_version(3.7)

# Default value is not to build test cases
if(NOT BUILD_TESTING)
    set(BUILD_TESTING OFF CACHE BOOL "BUILD_TESTING")
endif(NOT BUILD_TESTING)

# Default value is to build in release mode
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE INTERNAL "CMAKE_BUILD_TYPE")
endif(NOT CMAKE_BUILD_TYPE)

# Default value is to do update checks
if(NOT CRYFS_UPDATE_CHECKS)
    set(CRYFS_UPDATE_CHECKS ON CACHE BOOL "CRYFS_UPDATE_CHECKS")
endif(NOT CRYFS_UPDATE_CHECKS)

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(doc)
add_subdirectory(test)
add_subdirectory(cpack)
