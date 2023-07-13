# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

cmake_minimum_required(VERSION ${CRYPTOPP_MINIMUM_CMAKE_VERSION})

# Test project for cryptopp-cmake installation
project(CryptoppCmakeInstallPrefixTest)

# ---- Add dependencies via CPM ----
# see https://github.com/TheLartians/CPM.cmake for more info
include(${TEST_CMAKE_FILES_DIR}/CPM.cmake)

# ---- Speedup build using ccache (needs CPM) ----
include(${TEST_CMAKE_FILES_DIR}/FasterBuild.cmake)

set(CRYPTOPP_INCLUDE_PREFIX "crypto++")
cpmaddpackage(
  NAME
  cryptopp-cmake
  GIT_REPOSITORY
  https://github.com./abdes/cryptopp-cmake
  GIT_TAG
  master
  OPTIONS
  "CRYPTOPP_BUILD_TESTING OFF"
  "CRYPTOPP_INSTALL ON"
  "CRYPTOPP_INCLUDE_PREFIX ${CRYPTOPP_INCLUDE_PREFIX}")

add_executable(rng-test main.cpp)
target_link_libraries(rng-test PUBLIC cryptopp::cryptopp)
target_compile_features(rng-test PRIVATE cxx_constexpr)

include(${INT_TEST_CMAKE_FILES_DIR}/ProjectInstallTests.cmake)
