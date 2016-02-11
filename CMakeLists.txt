cmake_minimum_required(VERSION 3.0)

include(utils.cmake)

require_gcc_version(4.8)

add_subdirectory(vendor)
include_directories(vendor)

add_subdirectory(src)
add_subdirectory(test)
