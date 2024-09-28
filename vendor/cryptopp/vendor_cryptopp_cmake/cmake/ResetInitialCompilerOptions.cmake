# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

# This module is loaded by `cmake` while enabling support for each language from
# either the project() or enable_language() commands. It is loaded after CMake's
# builtin compiler and platform information modules have been loaded but before
# the information is used. The file may set platform information variables to
# override CMake's defaults.
#
# To load this module, set the variable `CMAKE_USER_MAKE_RULES_OVERRIDE` before
# you declare the project or enable a language:
# ~~~
# set(CMAKE_USER_MAKE_RULES_OVERRIDE "ResetInitialCompilerOptions")
# ~~~

# We use this module to strip compiler options that are not really needed but
# will cause compatibility issues with `ccache`.
if(MSVC AND USE_CCACHE)
    # As of ccache 4.6, /Zi option automatically added by cmake is unsupported.
    # Given that we are doing ccache only in development environments (USE_CCACHE
    # controls if ccache is enabled), we can just strip that option.
    macro(strip_unwanted_options_from cmake_flags)
        if(${cmake_flags} MATCHES "/Zi")
            string(REPLACE "/Zi" "/Z7" ${cmake_flags} ${${cmake_flags}})
        endif()
    endmacro()
    strip_unwanted_options_from(CMAKE_CXX_FLAGS_DEBUG_INIT)
    strip_unwanted_options_from(CMAKE_CXX_FLAGS_RELWITHDEBINFO_INIT)
    strip_unwanted_options_from(CMAKE_C_FLAGS_DEBUG_INIT)
    strip_unwanted_options_from(CMAKE_C_FLAGS_RELWITHDEBINFO_INIT)
endif()
