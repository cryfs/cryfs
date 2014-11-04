add_subdirectory(fuse)

add_library(cryfs_lib CryDevice.cpp)

target_link_libraries(cryfs_lib cryfs_fuse)
