add_library(cryfs_lib CryDevice.cpp CryDir.cpp CryFile.cpp CryNode.cpp CryOpenFile.cpp)

target_link_libraries(cryfs_lib boost_filesystem boost_system)
