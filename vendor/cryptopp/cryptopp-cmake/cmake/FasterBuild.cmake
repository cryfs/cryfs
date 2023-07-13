# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

# ------------------------------------------------------------------------------
# Reduce build time by using ccache when available
# ------------------------------------------------------------------------------

find_program(CCACHE_TOOL_PATH ccache)

if(NOT WIN32
   AND USE_CCACHE
   AND CCACHE_TOOL_PATH)
  message(STATUS "Using ccache (${CCACHE_TOOL_PATH}) (via wrapper).")
  # see https://github.com/TheLartians/Ccache.cmake enables CCACHE support
  # through the USE_CCACHE flag possible values are: YES, NO or equivalent
  include("${CMAKE_CURRENT_LIST_DIR}/CPM.cmake")
  cpmaddpackage("gh:TheLartians/Ccache.cmake@1.2.3")
elseif(
  WIN32
  AND USE_CCACHE
  AND CCACHE_TOOL_PATH)
  set(CMAKE_C_COMPILER_LAUNCHER
      ${CCACHE_TOOL_PATH}
      CACHE STRING "" FORCE)
  set(CMAKE_CXX_COMPILER_LAUNCHER
      ${CCACHE_TOOL_PATH}
      CACHE STRING "" FORCE)
  message(STATUS "Using ccache (${CCACHE_TOOL_PATH}).")
endif()
