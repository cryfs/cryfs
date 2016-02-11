include(CTest)

if (BUILD_TESTING)
  include_directories(../src)

  add_subdirectory(cpp-utils)
  add_subdirectory(fspp)
endif(BUILD_TESTING)
