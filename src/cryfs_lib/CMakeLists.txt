add_subdirectory(fusepp)

add_library(cryfs_lib CryDevice.cpp)

target_link_libraries(cryfs_lib fusepp)
