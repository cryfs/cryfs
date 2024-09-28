# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

include(${CPM_cryptopp-cmake_SOURCE}/cmake/cmake_minimum_required.cmake)
cmake_minimum_required(VERSION ${CRYPTOPP_MINIMUM_CMAKE_VERSION})

# Test project for the standard way of using cryptopp-cmake
project(CryptoppCmakeStandardTest)

# ---- Add dependencies via CPM ----
# see https://github.com/TheLartians/CPM.cmake for more info
include(${TEST_CMAKE_FILES_DIR}/CPM.cmake)

# ---- Speedup build using ccache (needs CPM) ----
include(${TEST_CMAKE_FILES_DIR}/FasterBuild.cmake)

cpmaddpackage(
  NAME
  cryptopp-cmake
  GIT_REPOSITORY
  abdes/cryptopp-cmake
  GIT_TAG
  ${cryptopp_GIT_BRANCH}
  OPTIONS
  "CRYPTOPP_BUILD_TESTING OFF")

# compile and link a test program using crypto++
add_executable(rng-test ${TEST_EXAMPLE_SOURCES_DIR}/main.cpp)
target_link_libraries(rng-test cryptopp::cryptopp)
target_compile_features(rng-test PRIVATE cxx_constexpr)
