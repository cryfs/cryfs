cmake_minimum_required(VERSION 3.1 FATAL_ERROR)

# TODO Remove this deprecated policy switch once we're on cmake 3.4 or later
cmake_policy(SET CMP0065 OLD)
cmake_policy(SET CMP0074 NEW)

# TODO Perf test:
#  - try if setting CRYPTOPP_NATIVE_ARCH=ON and adding -march=native to the compile commands for cryfs source files makes a difference
#    -> if yes, offer a cmake option to enable both of these

project(cryfs)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_LIST_DIR}/cmake-utils)
include(utils)

require_gcc_version(5.0)
require_clang_version(4.0)

# Default value is not to build test cases
option(BUILD_TESTING "build test cases" OFF)
option(CRYFS_UPDATE_CHECKS "let cryfs check for updates and security vulnerabilities" ON)
option(DISABLE_OPENMP "allow building without OpenMP libraries. This will cause performance degradations." OFF)

# The following options are helpful for development and/or CI
option(USE_WERROR "build with -Werror flag")
option(USE_CLANG_TIDY "build with clang-tidy checks enabled" OFF)
option(USE_IWYU "build with iwyu checks enabled" OFF)
option(CLANG_TIDY_WARNINGS_AS_ERRORS "treat clang-tidy warnings as errors" OFF)

set(CONAN_BUILD_INFO_FILE ${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)

if(EXISTS ${CONAN_BUILD_INFO_FILE})
    include(${CONAN_BUILD_INFO_FILE})
else()
    message(WARNING "The file ${CONAN_BUILD_INFO_FILE} doesn't exist, you have to run conan install first")
endif()
conan_basic_setup(TARGETS SKIP_STD)

if(CONAN_SETTINGS_COMPILER_LIBCXX STREQUAL "libstdc++")
    # TODO Test this warning works correctly and that the proposed solution in the warning message works.
    message(FATAL_ERROR "Conan is set up to build against libstdc++ (i.e. the legacy GCC ABI). We only support libstdc++11 (i.e. the new GCC ABI).\nPlease add the '-s compiler.libcxx=libstdc++11' argument when running 'conan install'.")
endif()

if(USE_IWYU)
    # note: for iwyu, we need cmake 3.3
    cmake_minimum_required(VERSION 3.3 FATAL_ERROR)
endif()
if(USE_CLANG_TIDY)
    # note: for clang-tidy, we need cmake 3.6, or (if the return code should be handled correctly, e.g. on CI), we need 3.8.
    cmake_minimum_required(VERSION 3.8 FATAL_ERROR)
endif()


if (MSVC)
    option(DOKAN_PATH "Location of the Dokan library, e.g. C:\\Program Files\\Dokan\\DokanLibrary-1.1.0" "")
endif()

# Default value is to build in release mode but with debug symbols
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE RelWithDebInfo CACHE INTERNAL "CMAKE_BUILD_TYPE")
endif(NOT CMAKE_BUILD_TYPE)

# The MSVC version on AppVeyor CI needs this
if(MSVC)
  add_definitions(/bigobj)
endif()

if (APPLE)
   add_compile_options ( -Wno-old-style-cast )
endif(APPLE)

add_subdirectory(vendor EXCLUDE_FROM_ALL)
add_subdirectory(src)
add_subdirectory(doc)
add_subdirectory(test)
add_subdirectory(cpack)
