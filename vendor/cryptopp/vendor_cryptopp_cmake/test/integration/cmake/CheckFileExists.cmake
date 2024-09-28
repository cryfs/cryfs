# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

# Helper script to check if a file exists at build time

message(STATUS "Checking if installed file \"${FILE_TO_CHECK}\" exists")
if(NOT EXISTS ${FILE_TO_CHECK})
  message(FATAL_ERROR "\"${FILE_TO_CHECK}\" doesn't exist.")
endif()
