# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

include(FetchContent)
set(version_underscore
    "${cryptopp-cmake_VERSION_MAJOR}_${cryptopp-cmake_VERSION_MINOR}_${cryptopp-cmake_VERSION_PATCH}"
)
if(GIT_FOUND)
    if(${CRYPTOPP_USE_MASTER_BRANCH})
        set(source_location "master")
    else()
        set(source_location "CRYPTOPP_${version_underscore}")
    endif()
    fetchcontent_declare(
        cryptopp
        GIT_REPOSITORY ${cryptopp-cmake_HOMEPAGE_URL}
        GIT_TAG ${source_location}
        QUIET
        SOURCE_DIR
        ${CRYPTOPP_INCLUDE_PREFIX}
    )
else()
    message(STATUS "Downloading crypto++ from URL...")
    cmake_policy(SET CMP0135 NEW)
    set(source_location "${cryptopp-cmake_HOMEPAGE_URL}/")
    if(NOT ${CRYPTOPP_USE_MASTER_BRANCH})
        string(
            APPEND
            source_location
            "releases/download/CRYPTOPP_${version_underscore}/cryptopp${cryptopp-cmake_VERSION_MAJOR}${cryptopp-cmake_VERSION_MINOR}${cryptopp-cmake_VERSION_PATCH}"
        )
    else()
        string(APPEND source_location "archive/refs/heads/master")
    endif()
    fetchcontent_declare(
        cryptopp
        URL "${source_location}.zip" QUIET SOURCE_DIR ${CRYPTOPP_INCLUDE_PREFIX}
    )
endif()
fetchcontent_populate(cryptopp)
