cmake_minimum_required(VERSION 3.25 FATAL_ERROR)

# TODO Perf test:
#  - try if setting CRYPTOPP_NATIVE_ARCH=ON and adding -march=native to the compile commands for cryfs source files makes a difference
#    -> if yes, offer a cmake option to enable both of these

project(cryfs)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_LIST_DIR}/cmake-utils)
include(utils)

require_gcc_version(7.0)
require_clang_version(7.0)

# Default value is not to build test cases
option(BUILD_TESTING "build test cases" OFF)
option(CRYFS_UPDATE_CHECKS "let cryfs check for updates and security vulnerabilities" ON)
option(DISABLE_OPENMP "allow building without OpenMP libraries. This will cause performance degradations." OFF)
option(USE_SYSTEM_LIBS "build with system libs instead of bundled libs." OFF)

# The following options are helpful for development and/or CI
option(USE_WERROR "build with -Werror flag")
option(USE_CLANG_TIDY "build with clang-tidy checks enabled" OFF)
option(USE_IWYU "build with iwyu checks enabled" OFF)
option(CLANG_TIDY_WARNINGS_AS_ERRORS "treat clang-tidy warnings as errors" OFF)

if (MSVC)
  option(DOKAN_PATH "Location of the Dokan library, e.g. C:\\Program Files\\Dokan\\DokanLibrary-2.2.0" "")
endif()

# Default value is to build in release mode but with debug symbols
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE RelWithDebInfo CACHE INTERNAL "CMAKE_BUILD_TYPE")
endif()

# We don't use LTO because crypto++ has problems with it, see https://github.com/weidai11/cryptopp/issues/1031 and https://www.cryptopp.com/wiki/Link_Time_Optimization

if(MSVC)
  # Needed to avoid a msvc build error "fatal error C1128: number of sections exceeded object file format limit: compile with /bigobj"
  add_definitions(/bigobj)
endif()

cmake_policy(SET CMP0167 NEW)
include(cmake-utils/Dependencies.cmake)

if(USE_SYSTEM_LIBS)
    include(FindPkgConfig)
    set(CMAKE_PREFIX_PATH ${CMAKE_PREFIX_PATH} /usr/lib /usr/lib64)
    pkg_check_modules(CRYPTOPP REQUIRED libcryptopp>=8.2)
    if(BUILD_TESTING)
        find_package(GTest CONFIG REQUIRED)
        set(GOOGLETEST_LIBS GTest::gtest GTest::gmock)
    endif()
    add_definitions(-DUSE_SYSTEM_LIBS)
else()
    add_subdirectory(vendor EXCLUDE_FROM_ALL)
    set(GOOGLETEST_LIBS googletest)
endif()

add_subdirectory(src)
add_subdirectory(doc)
add_subdirectory(test)
add_subdirectory(cpack)
