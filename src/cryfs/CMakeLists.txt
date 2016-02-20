project (cryfs)

set(SOURCES
    cli/Cli.cpp
    cli/VersionChecker.cpp
    cli/VersionCompare.cpp
    cli/CallAfterTimeout.cpp
    cli/program_options/utils.cpp
    cli/program_options/ProgramOptions.cpp
    cli/program_options/Parser.cpp
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

add_library(${PROJECT_NAME}_lib STATIC ${SOURCES})
target_link_libraries(${PROJECT_NAME}_lib PUBLIC cpp-utils blockstore blobstore fspp)
target_add_boost(${PROJECT_NAME}_lib program_options chrono)
target_enable_style_warnings(${PROJECT_NAME}_lib)
target_activate_cpp14(${PROJECT_NAME}_lib)
target_git_version_init(${PROJECT_NAME}_lib)

add_executable(${PROJECT_NAME} main.cpp)
target_link_libraries(${PROJECT_NAME} ${PROJECT_NAME}_lib)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

install(TARGETS cryfs
        COMPONENT cryfs-cli
        DESTINATION bin
        CONFIGURATIONS Release)
