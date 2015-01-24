add_library(blockstore_utils Data.cpp Key.cpp FileDoesntExistException.cpp BlockStoreUtils.cpp)

target_link_libraries(blockstore_utils cryptopp)
