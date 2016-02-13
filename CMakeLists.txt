cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

include(utils.cmake)

require_gcc_version(4.8)
require_clang_version(3.5) # clang 3.5 needed for spdlog dependency

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(cpack)
