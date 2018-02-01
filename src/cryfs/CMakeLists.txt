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
        filesystem/CrySymlink.cpp
        filesystem/CryDir.cpp
        filesystem/CryFile.cpp
        filesystem/CryDevice.cpp
        localstate/LocalStateDir.cpp
        localstate/LocalStateMetadata.cpp
        localstate/BasedirMetadata.cpp
)

add_library(${PROJECT_NAME} STATIC ${LIB_SOURCES})
set_target_properties(${PROJECT_NAME} PROPERTIES OUTPUT_NAME cryfs)
target_link_libraries(${PROJECT_NAME} PRIVATE cpp-utils blockstore blobstore fspp gitversion) # TODO Check that dependent projects don't get this linked in
target_add_boost(${PROJECT_NAME} program_options chrono) # TODO Check that dependent projects don't get boost added (use PRIVATE here)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

#install(TARGETS ${PROJECT_NAME}
#        DESTINATION lib
#        CONFIGURATIONS Release
#)
