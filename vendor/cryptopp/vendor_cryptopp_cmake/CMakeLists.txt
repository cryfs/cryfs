# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

# ------------------------------------------------------------------------------
# CMake basic options
# ------------------------------------------------------------------------------

include(${CMAKE_CURRENT_LIST_DIR}/cmake/cmake_minimum_required.cmake)
cmake_minimum_required(VERSION ${CRYPTOPP_MINIMUM_CMAKE_VERSION})

# List of directories specifying a search path for CMake modules to be loaded by
# the include() or find_package() commands before checking the default modules
# that come with CMake.
list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

# Setup our override file, in which we may cleanup cmake's default compiler
# options, based on what we are doing.
set(CMAKE_USER_MAKE_RULES_OVERRIDE "ResetInitialCompilerOptions")

# ------------------------------------------------------------------------------
# Project description and (meta) information
# ------------------------------------------------------------------------------

# Get git revision
include(GetGitRevisionDescription)
get_git_head_revision(GIT_REFSPEC GIT_SHA1)
string(SUBSTRING "${GIT_SHA1}" 0 12 GIT_REV)
if(NOT GIT_SHA1)
    set(GIT_REV "0")
endif()

set(META_PROJECT_NAME "cryptopp-cmake")
set(META_PROJECT_DESCRIPTION "Free C++ class library of cryptographic schemes")
set(META_GITHUB_REPO "https://github.com/weidai11/cryptopp")
set(META_VERSION_MAJOR "8")
set(META_VERSION_MINOR "9")
set(META_VERSION_PATCH "0")
set(META_VERSION_REVISION "${GIT_REV}")
set(META_VERSION
    "${META_VERSION_MAJOR}.${META_VERSION_MINOR}.${META_VERSION_PATCH}"
)
set(META_NAME_VERSION
    "${META_PROJECT_NAME} v${META_VERSION} (${META_VERSION_REVISION})"
)
# ------------------------------------------------------------------------------
# Project Declaration
# ------------------------------------------------------------------------------

# Declare project
project(
    "${META_PROJECT_NAME}"
    VERSION "${META_VERSION}"
    DESCRIPTION "${META_PROJECT_DESCRIPTION}"
    HOMEPAGE_URL "${META_GITHUB_REPO}"
    LANGUAGES CXX C ASM
)

# cmake-format: on

message("=> Project : ${PROJECT_NAME} v${cryptopp-cmake_VERSION}")

# ============================================================================
# Settable options
# ============================================================================

function(cryptopp_deprecated_option old_name new_name description default)
    if(DEFINED ${old_name})
        message(
            DEPRECATION
            "${old_name} is a deprecated option and will be ignored after 8.8.x + 3. Use ${new_name} instead"
        )
        set(default ${${old_name}})
    endif()

    option(${new_name} ${description} ${default})
endfunction()
option(
    CRYPTOPP_USE_INTERMEDIATE_OBJECTS_TARGET
    "Use a common intermediate objects target for the static and shared library targets"
    ON
)

if(CRYPTOPP_INCLUDE_PREFIX)
    set(CRYPTOPP_INCLUDE_PREFIX
        ${CRYPTOPP_INCLUDE_PREFIX}
        CACHE STRING
        "Set the dir where the headers get installed. Defaults to cryptopp."
    )
else()
    set(CRYPTOPP_INCLUDE_PREFIX
        "cryptopp"
        CACHE STRING
        "Set the dir where the headers get installed. Defaults to cryptopp."
    )
endif()

if(CRYPTOPP_SOURCES)
    set(CRYPTOPP_SOURCES
        ${CRYPTOPP_SOURCES}
        CACHE PATH
        "Use the provided location for crypto++ sources; do not fetch"
    )
else()
    set(CRYPTOPP_SOURCES
        ""
        CACHE PATH
        "Use the provided location for crypto++ sources; do not fetch"
    )
endif()

cryptopp_deprecated_option(
  USE_OPENMP
  CRYPTOPP_USE_OPENMP
  "Enable OpenMP to parallelize some of the algorithms. Note that this isn't always faster, see https://www.cryptopp.com/wiki/OpenMP"
  OFF
)

# These are IA-32 options.
cryptopp_deprecated_option(DISABLE_ASM CRYPTOPP_DISABLE_ASM "Disable ASM" OFF)
cryptopp_deprecated_option(DISABLE_SSSE3 CRYPTOPP_DISABLE_SSSE3 "Disable SSSE3" OFF)
cryptopp_deprecated_option(DISABLE_SSE4 CRYPTOPP_DISABLE_SSE4 "Disable SSE4" OFF)
cryptopp_deprecated_option(DISABLE_AESNI CRYPTOPP_DISABLE_AESNI "Disable AES-NI" OFF)
cryptopp_deprecated_option(DISABLE_CLMUL CRYPTOPP_DISABLE_CLMUL "Disable CLMUL" OFF)
cryptopp_deprecated_option(DISABLE_SHA CRYPTOPP_DISABLE_SHA "Disable SHA" OFF)
cryptopp_deprecated_option(DISABLE_AVX CRYPTOPP_DISABLE_AVX "Disable AVX" OFF)
cryptopp_deprecated_option(DISABLE_AVX2 CRYPTOPP_DISABLE_AVX2 "Disable AVX2" OFF)

# These are ARM A-32 options
cryptopp_deprecated_option(DISABLE_ARM_NEON CRYPTOPP_DISABLE_ARM_NEON "Disable NEON" OFF)

# These are Aarch64 options
cryptopp_deprecated_option(DISABLE_ARM_AES CRYPTOPP_DISABLE_ARM_AES "Disable ASIMD" OFF)
cryptopp_deprecated_option(DISABLE_ARM_AES CRYPTOPP_DISABLE_ARM_AES "Disable AES" OFF)
cryptopp_deprecated_option(DISABLE_ARM_PMULL CRYPTOPP_DISABLE_ARM_PMULL "Disable PMULL" OFF)
cryptopp_deprecated_option(DISABLE_ARM_SHA CRYPTOPP_DISABLE_ARM_SHA "Disable SHA" OFF)

# These are PowerPC options
cryptopp_deprecated_option(DISABLE_ALTIVEC CRYPTOPP_DISABLE_ALTIVEC "Disable Altivec" OFF)
cryptopp_deprecated_option(DISABLE_POWER7 CRYPTOPP_DISABLE_POWER7 "Disable POWER7" OFF)
cryptopp_deprecated_option(DISABLE_POWER8 CRYPTOPP_DISABLE_POWER8 "Disable POWER8" OFF)
cryptopp_deprecated_option(DISABLE_POWER9 CRYPTOPP_DISABLE_POWER9 "Disable POWER9" OFF)

option(
    CRYPTOPP_USE_MASTER_BRANCH
    "Get crypto++ from the master branch, not from a release tag"
    FALSE
)
option(CRYPTOPP_BUILD_TESTING "Build library tests" ON)
option(
    CRYPTOPP_BUILD_DOCUMENTATION
    "Use Doxygen to create the HTML-based API documentation"
    OFF
)

# Override the CRYPTOPP_INSTALL option to ON/OFF to respectively force
# install/no-install behavior for cryptopp module. This is particularly useful
# when `cryptopp` is used as a sub-project with CMake and the user publicly
# depends on it and wants to have a self-contained install.
option(CRYPTOPP_INSTALL "Generate the install target for this project." ON)

# Crypto++ DOES NOT properly support DLL builds. The old DLL was the FIPS one,
# which is being abandoned. Therefore, we only allow static builds until that
# situation is solved.
#
# See https://cryptopp.com/wiki/Wrapper_DLL for a workaround.
option(CRYPTOPP_BUILD_SHARED "Build shared library" OFF)
if(${CRYPTOPP_BUILD_SHARED})
    message(
        FATAL_ERROR
        "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n"
        "Crypto++ DOES NOT properly support DLL builds. The old DLL was the FIPS"
        "one which is being abandoned.\nTherefore, we only allow static builds"
        "until that situation changes.\n"
        "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n"
    )
endif()

if(EXISTS "${CMAKE_SOURCE_DIR}/.git")
    set(CRYPTOPP_DEFAULT_BUILD_TYPE "Debug")
else()
    set(CRYPTOPP_DEFAULT_BUILD_TYPE "Release")
endif(EXISTS "${CMAKE_SOURCE_DIR}/.git")

# Set the possible values of build type for cmake-gui
if(CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_CONFIGURATION_TYPES
        "Debug;Release"
        CACHE STRING
        "Semicolon separated list of supported configuration types, only supports debug and release, anything else will be ignored"
        FORCE
    )

    set_property(
        CACHE CMAKE_CONFIGURATION_TYPES
        PROPERTY STRINGS "Debug" "Release"
    )
endif()

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    message(
        STATUS
        "Setting build type to '${CRYPTOPP_DEFAULT_BUILD_TYPE}' as none was specified."
    )

    set(CMAKE_BUILD_TYPE
        "${CRYPTOPP_DEFAULT_BUILD_TYPE}"
        CACHE STRING
        "Choose the type of build."
        FORCE
    )
endif()

# ---- Speedup build using ccache (needs CPM) ----
include(cmake/FasterBuild.cmake)

# ------------------------------------------------------------------------------
# Fetch / Find crypto++
# ------------------------------------------------------------------------------

# If CMake is invoked with an explicit option (cryptopp_SOURCE_DIR), setting
# the location for user-provided sources of crypto++, or if the automatic
# download fails, we use that option value to find the sources.
find_package(Git)

if(NOT CRYPTOPP_SOURCES)
    include(GetCryptoppSources)
    if(EXISTS ${cryptopp_SOURCE_DIR})
        message(STATUS "Crypto++ auto fetched at: ${cryptopp_SOURCE_DIR}")
    else()
        message(FATAL_ERROR "Crypto++ auto fetch failed; cannot continue!")
    endif()
else()
    if(EXISTS ${CRYPTOPP_SOURCES})
        message(
            STATUS
            "Crypto++ from user-specified location at: ${CRYPTOPP_SOURCES}"
        )
        set(cryptopp_SOURCE_DIR ${CRYPTOPP_SOURCES})
    else()
        message(
            FATAL_ERROR
            "User-provided location (${CRYPTOPP_SOURCES}) for crypto++ sources does not exit!"
        )
    endif()
endif()

if(Git_FOUND)
    get_filename_component(
        CRYPTOPP_CMAKE_PARENT_DIR
        ${cryptopp-cmake_SOURCE_DIR}
        DIRECTORY
    )
    execute_process(
        COMMAND
            ${CMAKE_COMMAND} -E env
            GIT_CEILING_DIRECTORIES="${CRYPTOPP_CMAKE_PARENT_DIR}"
            ${GIT_EXECUTABLE} rev-parse --abbrev-ref HEAD
        WORKING_DIRECTORY ${cryptopp-cmake_SOURCE_DIR}
        RESULT_VARIABLE cryptopp_CMAKE_GIT_RESULT
        OUTPUT_VARIABLE cryptopp_GIT_BRANCH
        ERROR_QUIET
    )
    if(cryptopp_CMAKE_GIT_RESULT EQUAL 0)
        string(STRIP ${cryptopp_GIT_BRANCH} cryptopp_GIT_BRANCH)
    endif()
endif()
if(NOT cryptopp_GIT_BRANCH)
    set(cryptopp_GIT_BRANCH "master")
endif()
message(STATUS "Using branch ${cryptopp_GIT_BRANCH} for tests")

# ------------------------------------------------------------------------------
# Testing
# ------------------------------------------------------------------------------

if(CRYPTOPP_BUILD_TESTING)
    enable_testing()
    add_subdirectory(test)
endif()

# ------------------------------------------------------------------------------
# Add cryptopp CMake subdirectory
# ------------------------------------------------------------------------------

message("=> Module : cryptopp")
add_subdirectory(cryptopp)
