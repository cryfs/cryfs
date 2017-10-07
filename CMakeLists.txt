cmake_minimum_required(VERSION 2.8 FATAL_ERROR)

# Determine OS
exec_program(uname OUTPUT_VARIABLE SYSTEM_NAME)
SET(SYSTEM_NAME "${SYSTEM_NAME}" CACHE INTERNAL "")

project(cryfs)

include(utils.cmake)

require_gcc_version(4.8)
require_clang_version(3.7)

# Default value is not to build test cases
if(NOT BUILD_TESTING)
    set(BUILD_TESTING OFF CACHE BOOL "BUILD_TESTING")
endif(NOT BUILD_TESTING)

# Default vaule is to build in release mode
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

if(SYSTEM_NAME STREQUAL "FreeBSD")
  # use a linker from ports, e.g. lld50 from llvm
  message("use a linker built from ports")

  if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "Clang") # using Clang
    # Ensure FreeBSD build uses current clang >= from ports
    # which is required to correctly link the required libraries.
    # Since FreeBSD 11.1 bundles clang 4.0.0, require at least 5.0
    message("you have to use clang built from ports")
    require_clang_version(5.0)
  else()
    message(FATAL_ERROR "clang >= 5.0 from ports required")
  endif()
endif(SYSTEM_NAME STREQUAL "FreeBSD")
