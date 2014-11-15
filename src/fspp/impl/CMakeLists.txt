add_library(fspp_impl FilesystemImpl.cpp)

target_link_libraries(fspp_impl boost_filesystem boost_system)
