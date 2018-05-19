include(CTest)

if (BUILD_TESTING)
  add_subdirectory(gtest-1.8.0 EXCLUDE_FROM_ALL)

  project (googletest)
  add_library(${PROJECT_NAME} dummy.cpp)
  target_link_libraries(${PROJECT_NAME} PRIVATE gtest gmock gmock_main)
  target_include_directories(${PROJECT_NAME} SYSTEM INTERFACE ${gtest_INCLUDE_DIRS}/include SYSTEM ${gmock_INCLUDE_DIRS}/include)

  # Disable "missing override" warning because gmock MOCK_METHOD() don't use override :(
  target_compile_options(${PROJECT_NAME} PUBLIC "-Wno-inconsistent-missing-override")
endif (BUILD_TESTING)
