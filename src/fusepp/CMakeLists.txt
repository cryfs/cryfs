add_library(fusepp Fuse.cpp)

target_link_libraries(fusepp fuse boost_filesystem boost_system)
