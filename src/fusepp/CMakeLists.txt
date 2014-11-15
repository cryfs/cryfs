add_library(fusepp Fuse.cpp FuseDevice.cpp FuseDir.cpp FuseErrnoException.cpp FuseFile.cpp FuseNode.cpp FuseOpenFile.cpp FuseOpenFileList.cpp IdList.cpp)

target_link_libraries(fusepp fuse boost_filesystem boost_system)
