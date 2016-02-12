cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

include(utils.cmake)

require_gcc_version(4.8)

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(cpack)
