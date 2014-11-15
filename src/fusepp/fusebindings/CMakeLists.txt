add_library(fusepp_fuse_bindings Fuse.cpp)

target_link_libraries(fusepp_fuse_bindings fuse fusepp_impl)
