# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

include(GNUInstallDirs)

# add custom target to check the installed files at build time
add_custom_target(do-checks)

function(check_file_exists file_to_check)
  add_custom_command(
    TARGET do-checks
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -DFILE_TO_CHECK=${file_to_check} -P
            ${CMAKE_CURRENT_LIST_DIR}/CheckFileExists.cmake
    COMMENT "Checking if ${file_to_check} exists...")
endfunction()

if(MSVC)
  if(CRYPTOPP_BUILD_SHARED)
    check_file_exists(${CMAKE_INSTALL_FULL_LIBDIR}/cryptopp.dll)
  else()
    check_file_exists(${CMAKE_INSTALL_FULL_LIBDIR}/cryptopp.lib)
  endif()
else()
  check_file_exists(${CMAKE_INSTALL_FULL_LIBDIR}/$<TARGET_FILE_NAME:cryptopp>)
endif()
check_file_exists(${CMAKE_INSTALL_FULL_INCLUDEDIR}/${CRYPTOPP_INCLUDE_PREFIX})
check_file_exists(
  ${CMAKE_INSTALL_FULL_INCLUDEDIR}/${CRYPTOPP_INCLUDE_PREFIX}/config.h)
check_file_exists(${CMAKE_INSTALL_FULL_DATAROOTDIR}/pkgconfig/cryptopp.pc)
check_file_exists(${CMAKE_INSTALL_FULL_DATAROOTDIR}/cmake/cryptopp)
check_file_exists(
  ${CMAKE_INSTALL_FULL_DATAROOTDIR}/cmake/cryptopp/cryptoppConfig.cmake)
