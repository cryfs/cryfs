include(CTest)

if (BUILD_TESTING)
  include_directories(../src)

  add_subdirectory(cpp-utils)
  add_subdirectory(fspp)
  add_subdirectory(parallelaccessstore)
endif(BUILD_TESTING)
