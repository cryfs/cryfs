# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

set(TEST_CMAKE_FILES_DIR ${CMAKE_SOURCE_DIR}/cmake)
set(TEST_EXAMPLE_SOURCES_DIR ${CMAKE_CURRENT_SOURCE_DIR}/example-src)

# set(FETCHCONTENT_BASE_DIR ${CMAKE_CURRENT_BINARY_DIR}) cmake_path(APPEND
# FETCHCONTENT_BASE_DIR _deps)

add_subdirectory(unit)
add_subdirectory(integration)

add_custom_target(
  test-verbose
  WORKING_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/.."
  COMMAND ${CMAKE_CTEST_COMMAND} --verbose --output-on-failure)
