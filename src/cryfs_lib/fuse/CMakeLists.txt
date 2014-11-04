add_library(cryfs_fuse Fuse.cpp)

target_link_libraries(cryfs_fuse fuse)
