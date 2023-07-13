# ===-----------------------------------------------------------------------===#
# Distributed under the MIT License (https://opensource.org/licenses/MIT).
# SPDX-License-Identifier: MIT
#
# See details at:
# https://alexreinking.com/blog/building-a-dual-shared-and-static-library-with-cmake.html
# and full source at https://github.com/alexreinking/SharedStaticStarter
# ===-----------------------------------------------------------------------===#

cmake_minimum_required(VERSION 3.12)

set(cryptopp_known_comps static shared)
set(cryptopp_comp_static NO)
set(cryptopp_comp_shared NO)
foreach(cryptopp_comp IN LISTS ${CMAKE_FIND_PACKAGE_NAME}_FIND_COMPONENTS)
  if(cryptopp_comp IN_LIST cryptopp_known_comps)
    set(cryptopp_comp_${cryptopp_comp} YES)
  else()
    set(${CMAKE_FIND_PACKAGE_NAME}_NOT_FOUND_MESSAGE
        "cryptopp does not recognize component `${cryptopp_comp}`.")
    set(${CMAKE_FIND_PACKAGE_NAME}_FOUND FALSE)
    return()
  endif()
endforeach()

if(cryptopp_comp_static AND cryptopp_comp_shared)
  set(${CMAKE_FIND_PACKAGE_NAME}_NOT_FOUND_MESSAGE
      "cryptopp `static` and `shared` components are mutually exclusive.")
  set(${CMAKE_FIND_PACKAGE_NAME}_FOUND FALSE)
  return()
endif()

set(cryptopp_static_targets
    "${CMAKE_CURRENT_LIST_DIR}/cryptopp-static-targets.cmake")
set(cryptopp_shared_targets
    "${CMAKE_CURRENT_LIST_DIR}/cryptopp-shared-targets.cmake")

macro(cryptopp_load_targets type)
  if(NOT EXISTS "${cryptopp_${type}_targets}")
    set(${CMAKE_FIND_PACKAGE_NAME}_NOT_FOUND_MESSAGE
        "cryptopp `${type}` libraries were requested but not found.")
    set(${CMAKE_FIND_PACKAGE_NAME}_FOUND FALSE)
    return()
  endif()
  include("${cryptopp_${type}_targets}")
endmacro()

if(cryptopp_comp_static)
  cryptopp_load_targets(static)
elseif(cryptopp_comp_shared)
  cryptopp_load_targets(shared)
elseif(DEFINED cryptopp_SHARED_LIBS AND cryptopp_SHARED_LIBS)
  cryptopp_load_targets(shared)
elseif(DEFINED cryptopp_SHARED_LIBS AND NOT cryptopp_SHARED_LIBS)
  cryptopp_load_targets(static)
elseif(BUILD_SHARED_LIBS)
  if(EXISTS "${cryptopp_shared_targets}")
    cryptopp_load_targets(shared)
  else()
    cryptopp_load_targets(static)
  endif()
else()
  if(EXISTS "${cryptopp_static_targets}")
    cryptopp_load_targets(static)
  else()
    cryptopp_load_targets(shared)
  endif()
endif()
