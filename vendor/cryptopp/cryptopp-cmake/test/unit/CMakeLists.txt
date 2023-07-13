# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

file(GLOB tests "${CMAKE_CURRENT_SOURCE_DIR}/*")
list(REMOVE_ITEM tests "${CMAKE_CURRENT_SOURCE_DIR}/CMakeLists.txt")

foreach(test ${tests})
  cmake_path(GET test FILENAME test_name)
  message(STATUS "Adding unit test: ${test_name}")
  # Configure
  add_test(
    NAME cryptopp-${test_name}-configure
    COMMAND
      ${CMAKE_COMMAND}
      # Always use ninja, never the visual studio generator (bugged)
      -G${CMAKE_GENERATOR}
      # Pass the locations for common cmake files
      -D "TEST_CMAKE_FILES_DIR=${TEST_CMAKE_FILES_DIR}"
      # Pass the locations for common test source files
      -D "TEST_EXAMPLE_SOURCES_DIR=${TEST_EXAMPLE_SOURCES_DIR}"
      # Use ccache
      -D "USE_CCACHE=TRUE"
      # Override cmake compiler flags for ccache/MSVC
      -D
      "CMAKE_USER_MAKE_RULES_OVERRIDE=${TEST_CMAKE_FILES_DIR}/ResetInitialCompilerOptions.cmake"
      # Setup cmake source/build dirs
      -S "${test}" -B "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
      # Use local source code for cryptopp-cmake
      -D "CPM_cryptopp-cmake_SOURCE=${cryptopp-cmake_SOURCE_DIR}"
      # Enable verbose makefiles so we can see the full compile command line
      -D CMAKE_VERBOSE_MAKEFILE=ON
      # Set the build-type to what we are building
      -D CMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE})
  set_tests_properties(cryptopp-${test_name}-configure PROPERTIES
                       FIXTURES_SETUP ${test_name}-config
                       LABELS "cryptopp;cryptopp-unit-tests;cryptopp-${test_name}")
  # Build
  add_test(NAME cryptopp-${test_name}-build
           COMMAND ${CMAKE_COMMAND} --build
                   "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
                   --config ${CMAKE_BUILD_TYPE})
  # Run build test case after the configure test case
  set_tests_properties(cryptopp-${test_name}-build PROPERTIES
                       FIXTURES_REQUIRED ${test_name}-config
                       LABELS "cryptopp;cryptopp-unit-tests;cryptopp-${test_name}")
endforeach()
