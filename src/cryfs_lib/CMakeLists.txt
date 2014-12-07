add_library(cryfs_lib CryDevice.cpp CryDir.cpp CryFile.cpp CryNode.cpp CryOpenFile.cpp CryConfig.cpp impl/DirBlob.cpp)

target_link_libraries(cryfs_lib)
