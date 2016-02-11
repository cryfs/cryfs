# Earlier cmake versions generate .deb packages for which the package manager says they're bad quality
# and asks the user whether they really want to install it. Cmake 3.3 fixes this.
cmake_minimum_required(VERSION 3.3)

include(utils.cmake)

require_gcc_version(4.8)

add_subdirectory(vendor)
add_subdirectory(src)
add_subdirectory(test)
add_subdirectory(cpack)
