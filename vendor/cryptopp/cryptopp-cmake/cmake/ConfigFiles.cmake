# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

include(CMakePackageConfigHelpers)

# ------------------------------------------------------------------------------
# Generate module config files for cmake and pkgconfig
# ------------------------------------------------------------------------------
function(_module_cmake_config_files)
  message(STATUS "[cryptopp] Generating cmake package config files")
  write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/cryptoppConfigVersion.cmake
    COMPATIBILITY SameMajorVersion)
endfunction()

function(_module_pkgconfig_files)
  message(STATUS "[cryptopp] Generating pkgconfig files")
  set(MODULE_PKGCONFIG_FILE cryptopp.pc)

  if(CMAKE_BUILD_TYPE EQUAL "Debug")
    get_target_property(target_debug_postfix cryptopp DEBUG_POSTFIX)
    if(${target_debug_postfix} MATCHES "-NOTFOUND$")
      set(target_debug_postfix "")
    endif()
  endif()
  set(MODULE_LINK_LIBS "-lcryptopp${target_debug_postfix}")

  configure_file(config.pc.in
                 ${CMAKE_CURRENT_BINARY_DIR}/${MODULE_PKGCONFIG_FILE} @ONLY)
endfunction()

function(create_module_config_files)
  _module_cmake_config_files()
  _module_pkgconfig_files()
endfunction()
