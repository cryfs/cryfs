add_library(cryfs_lib CryDevice.cpp CryDir.cpp CryErrnoException.cpp CryFile.cpp CryNode.cpp)

target_link_libraries(cryfs_lib boost_filesystem)
