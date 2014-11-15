add_library(fspp_fuse Fuse.cpp)

target_link_libraries(fspp_fuse fuse fspp_impl)
