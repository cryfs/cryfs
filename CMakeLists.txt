cmake_minimum_required(VERSION 3.0 FATAL_ERROR)
cmake_policy(SET CMP0054 NEW)

# note: for clang-tidy, we need cmake 3.6, or (if the return code should be handled correctly, e.g. on CI), we need 3.8.

# TODO Perf test:
#  - try if setting CRYPTOPP_NATIVE_ARCH=ON and adding -march=native to the compile commands for cryfs source files makes a difference
#    -> if yes, offer a cmake option to enable both of these

project(cryfs)

include(cmake-utils/utils.cmake)

require_gcc_version(5.0)
require_clang_version(4.0)

# Default value is not to build test cases
option(BUILD_TESTING "build test cases" OFF)
option(CRYFS_UPDATE_CHECKS "let cryfs check for updates and security vulnerabilities" ON)
option(DISABLE_OPENMP "allow building without OpenMP libraries. This will cause performance degradations." OFF)
option(USE_CLANG_TIDY "build with clang-tidy checks enabled" OFF)
option(CLANG_TIDY_WARNINGS_AS_ERRORS "treat clang-tidy warnings as errors" OFF)

if (MSVC)
    option(DOKAN_PATH "Location of the Dokan library, e.g. C:\\Program Files\\Dokan\\DokanLibrary-1.1.0" "")
endif()

# Default value is to build in release mode
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE INTERNAL "CMAKE_BUILD_TYPE")
endif(NOT CMAKE_BUILD_TYPE)

# The MSVC version on AppVeyor CI needs this
if(MSVC)
  add_definitions(/bigobj)
endif()

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(doc)
add_subdirectory(test)
add_subdirectory(cpack)
