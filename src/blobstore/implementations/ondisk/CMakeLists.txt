add_library(blobstore_ondisk Data.cpp OnDiskBlob.cpp OnDiskBlobStore.cpp)

target_link_libraries(blobstore_ondisk boost_filesystem boost_system)
