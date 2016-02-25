project (cryfs-lib)

set(SOURCES
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
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})
set_target_properties(${PROJECT_NAME} PROPERTIES OUTPUT_NAME cryfs)
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils blockstore blobstore fspp)
target_add_boost(${PROJECT_NAME} program_options chrono)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
target_git_version_init(${PROJECT_NAME})

# Needed so it can be linked in to the shared cryfs library
set_target_properties(${PROJECT_NAME} PROPERTIES POSITION_INDEPENDENT_CODE on)


project(cryfs)

add_library(${PROJECT_NAME} SHARED cryfs.cpp)
target_link_libraries(${PROJECT_NAME} PRIVATE cryfs-lib) # TODO Check that dependent projects don't get this linked in
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

# The library should only export the functions we declared to be exported
include(GenerateExportHeader)
add_compiler_export_flags(EXPORT_FLAGS)
separate_arguments(EXPORT_FLAGS)
target_compile_options(${PROJECT_NAME} PRIVATE ${EXPORT_FLAGS})
generate_export_header(${PROJECT_NAME})
target_include_directories(${PROJECT_NAME} PUBLIC ${PROJECT_BINARY_DIR})

install(TARGETS ${PROJECT_NAME}
        DESTINATION lib
        CONFIGURATIONS Release
)
