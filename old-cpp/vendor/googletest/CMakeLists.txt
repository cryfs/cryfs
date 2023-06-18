include(CTest)

if (BUILD_TESTING)
  # Fix how gtest links the C library on windows, see https://github.com/google/googletest/blob/4e4df226fc197c0dda6e37f5c8c3845ca1e73a49/googletest/README.md
  set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
  set(INSTALL_GTEST off CACHE BOOL "" FORCE)
  set(INSTALL_GMOCK off CACHE BOOL "" FORCE)
  add_subdirectory(gtest EXCLUDE_FROM_ALL)
  
  add_library(googletest INTERFACE)
  target_link_libraries(googletest INTERFACE gtest gmock)
  target_include_directories(googletest SYSTEM INTERFACE ${gtest_INCLUDE_DIRS}/include SYSTEM INTERFACE ${gmock_INCLUDE_DIRS}/include)
endif()
