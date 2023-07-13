# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

set(INT_TEST_CMAKE_FILES_DIR ${CMAKE_CURRENT_SOURCE_DIR}/cmake)
set(CRYPTOPP_CMAKE_INSTALL_ROOT
    "${CMAKE_BINARY_DIR}/test-dirs/install")
define_property(TEST PROPERTY install_dir
                BRIEF_DOCS "Install-dir of the test"
                FULL_DOCS "Just set because not optional (yet)")

file(GLOB tests "${CMAKE_CURRENT_SOURCE_DIR}/int-install-*")
foreach(test ${tests})
  cmake_path(GET test FILENAME test_name)
  message(STATUS "Adding install integration test: ${test_name}")
  string (REPLACE "int-" "" short_name cryptopp-${test_name})

  # Configure
  add_test(
    NAME cryptopp-${test_name}-configure
    COMMAND
      ${CMAKE_COMMAND}
      -G${CMAKE_GENERATOR}
      # Pass the locations for common cmake test files
      -D "TEST_CMAKE_FILES_DIR=${TEST_CMAKE_FILES_DIR}"
      -D "INT_TEST_CMAKE_FILES_DIR=${INT_TEST_CMAKE_FILES_DIR}"
      # Pass the locations for common test source files
      -D "TEST_EXAMPLE_SOURCES_DIR=${TEST_EXAMPLE_SOURCES_DIR}"
      # Use ccache
      -D "USE_CCACHE=TRUE"
      # Override cmake compiler flags for ccache/MSVC
      -D
      "CMAKE_USER_MAKE_RULES_OVERRIDE=${TEST_CMAKE_FILES_DIR}/ResetInitialCompilerOptions.cmake"
      # Set the install prefix
      -D "CMAKE_INSTALL_PREFIX=${CRYPTOPP_CMAKE_INSTALL_ROOT}/${test_name}"
      # Setup cmake source/build dirs
      -S "${CMAKE_CURRENT_SOURCE_DIR}/${test_name}" -B
      "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
      # Use local source code for cryptopp-cmake
      -D "CPM_cryptopp-cmake_SOURCE=${CMAKE_CURRENT_SOURCE_DIR}/../.."
      # Enable verbose makefiles so we can see the full compile command line
      -D "CMAKE_VERBOSE_MAKEFILE=ON"
      # Throw the version in
      -D CRYPTOPP_MINIMUM_CMAKE_VERSION=${CRYPTOPP_MINIMUM_CMAKE_VERSION}
      # Set the build-type to what we are building
      -D CMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE})
  set_tests_properties(cryptopp-${test_name}-configure PROPERTIES
                       FIXTURES_SETUP ${test_name}-config
                       install_dir "${CRYPTOPP_CMAKE_INSTALL_ROOT}/${test_name}"
                       LABELS "cryptopp;cryptopp-integration-tests;${short_name}")

  # Build
  add_test(NAME cryptopp-${test_name}-build
           COMMAND ${CMAKE_COMMAND} --build
                   "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
                   --config ${CMAKE_BUILD_TYPE})
  set_tests_properties(cryptopp-${test_name}-build PROPERTIES
                       FIXTURES_SETUP ${test_name}-build
                       FIXTURES_REQUIRED ${test_name}-config
                       LABELS "cryptopp;cryptopp-integration-tests;${short_name}")

  # Install
  add_test(
    NAME cryptopp-${test_name}-install
    COMMAND ${CMAKE_COMMAND} --build
            "${CMAKE_BINARY_DIR}/test-dirs/${test_name}" --target install
            --config ${CMAKE_BUILD_TYPE})
  set_tests_properties(cryptopp-${test_name}-install PROPERTIES
                       FIXTURES_SETUP ${test_name}-install
                       FIXTURES_REQUIRED ${test_name}-build
                       LABELS "cryptopp;cryptopp-integration-tests;${short_name}")

  # Check installed files
  add_test(
    NAME cryptopp-${test_name}-checks
    COMMAND ${CMAKE_COMMAND} --build
            "${CMAKE_BINARY_DIR}/test-dirs/${test_name}" --target do-checks
            --config ${CMAKE_BUILD_TYPE})
  set_tests_properties(cryptopp-${test_name}-checks PROPERTIES
                       FIXTURES_REQUIRED ${test_name}-install
                       LABELS "cryptopp;cryptopp-integration-tests;${short_name}")
endforeach()

# ------------------------------------------------------------------------------
# Use cryptopp via find_package
# ------------------------------------------------------------------------------

set(test_name "int-find-package")
get_test_property(cryptopp-int-install-default-configure
                  install_dir CRYPTOPP_SEARCH_ROOT)

# Configure
add_test(
  NAME cryptopp-${test_name}-configure
  COMMAND
    ${CMAKE_COMMAND}
    -G${CMAKE_GENERATOR}
    # Pass the locations for common test source files
    -D "TEST_EXAMPLE_SOURCES_DIR=${TEST_EXAMPLE_SOURCES_DIR}"
    # Set the install prefix
    -D "CRYPTOPP_CMAKE_INSTALL_ROOT=${CRYPTOPP_CMAKE_INSTALL_ROOT}"
    # Setup cmake source/build dirs
    -S "${CMAKE_CURRENT_SOURCE_DIR}/${test_name}" -B
    "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
    # Enable verbose makefiles so we can see the full compile command line
    -D "CMAKE_VERBOSE_MAKEFILE=ON"
    # Throw the version in
    -D CRYPTOPP_MINIMUM_CMAKE_VERSION=${CRYPTOPP_MINIMUM_CMAKE_VERSION}
    # Set the build-type to what we are building
    -D CMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
    # Tell the test where to find installed package
    -D cryptopp_DIR=${CRYPTOPP_SEARCH_ROOT}/share/cmake/cryptopp)

set_tests_properties(cryptopp-${test_name}-configure PROPERTIES
                     FIXTURES_SETUP ${test_name}-config
                     FIXTURES_REQUIRED "int-install-default-install"
                     LABELS "cryptopp;cryptopp-find_package")

# Build
add_test(NAME cryptopp-${test_name}-build
         COMMAND ${CMAKE_COMMAND} --build
                 "${CMAKE_BINARY_DIR}/test-dirs/${test_name}"
                 --config ${CMAKE_BUILD_TYPE})
set_tests_properties(cryptopp-${test_name}-build PROPERTIES
                     FIXTURES_REQUIRED ${test_name}-config
                     LABELS "cryptopp;cryptopp-find_package")
