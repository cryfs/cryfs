project (cryfs)

set(LIB_SOURCES
#    cryfs.cpp
        CryfsException.cpp
        config/crypto/outer/OuterConfig.cpp
        config/crypto/outer/OuterEncryptor.cpp
        config/crypto/CryConfigEncryptorFactory.cpp
        config/crypto/inner/ConcreteInnerEncryptor.cpp
        config/crypto/inner/InnerConfig.cpp
        config/crypto/inner/InnerEncryptor.cpp
        config/crypto/CryConfigEncryptor.cpp
        config/CryConfigConsole.cpp
        config/CryConfigLoader.cpp
        config/CryConfig.cpp
        config/CryConfigFile.cpp
        config/CryCipher.cpp
        config/CryConfigCreator.cpp
        config/CryKeyProvider.cpp
        config/CryPasswordBasedKeyProvider.cpp
        config/CryPresetPasswordBasedKeyProvider.cpp
        filesystem/CryOpenFile.cpp
        filesystem/fsblobstore/utils/DirEntry.cpp
        filesystem/fsblobstore/utils/DirEntryList.cpp
        filesystem/fsblobstore/FsBlobStore.cpp
        filesystem/fsblobstore/FsBlobView.cpp
        filesystem/fsblobstore/FileBlob.cpp
        filesystem/fsblobstore/FsBlob.cpp
        filesystem/fsblobstore/SymlinkBlob.cpp
        filesystem/fsblobstore/DirBlob.cpp
        filesystem/CryNode.cpp
        filesystem/parallelaccessfsblobstore/DirBlobRef.cpp
        filesystem/parallelaccessfsblobstore/ParallelAccessFsBlobStore.cpp
        filesystem/parallelaccessfsblobstore/ParallelAccessFsBlobStoreAdapter.cpp
        filesystem/parallelaccessfsblobstore/FsBlobRef.cpp
        filesystem/parallelaccessfsblobstore/FileBlobRef.cpp
        filesystem/parallelaccessfsblobstore/SymlinkBlobRef.cpp
        filesystem/CrySymlink.cpp
        filesystem/CryDir.cpp
        filesystem/cachingfsblobstore/DirBlobRef.cpp
        filesystem/cachingfsblobstore/CachingFsBlobStore.cpp
        filesystem/cachingfsblobstore/FsBlobRef.cpp
        filesystem/cachingfsblobstore/FileBlobRef.cpp
        filesystem/cachingfsblobstore/SymlinkBlobRef.cpp
        filesystem/CryFile.cpp
        filesystem/CryDevice.cpp
        localstate/LocalStateDir.cpp
        localstate/LocalStateMetadata.cpp
        localstate/BasedirMetadata.cpp
)

add_library(${PROJECT_NAME} STATIC ${LIB_SOURCES})
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils fspp-interface blockstore blobstore gitversion)
target_add_boost(${PROJECT_NAME} program_options chrono) # TODO Check that dependent projects don't get boost added (use PRIVATE here)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

#install(TARGETS ${PROJECT_NAME}
#        DESTINATION lib
#        CONFIGURATIONS Release
#)
