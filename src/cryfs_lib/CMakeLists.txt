add_subdirectory(fusepp)

add_library(cryfs_lib CryDevice.cpp CryFuse.cpp)

target_link_libraries(cryfs_lib fusepp)
