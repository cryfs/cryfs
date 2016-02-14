cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

include(utils.cmake)

require_gcc_version(4.8)
require_clang_version(3.5) # clang 3.5 needed for spdlog dependency

# Default value is not to build test cases
if(NOT DEFINED BUILD_TESTING)
    set(BUILD_TESTING OFF CACHE INTERNAL "BUILD_TESTING")
endif(NOT DEFINED BUILD_TESTING)

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(cpack)
