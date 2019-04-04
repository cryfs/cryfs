project (cryfs-test)

set(SOURCES
        impl/config/crypto/CryConfigEncryptorFactoryTest.cpp
        impl/config/crypto/outer/OuterConfigTest.cpp
        impl/config/crypto/outer/OuterEncryptorTest.cpp
        impl/config/crypto/inner/ConcreteInnerEncryptorTest.cpp
        impl/config/crypto/inner/InnerConfigTest.cpp
        impl/config/crypto/CryConfigEncryptorTest.cpp
        impl/config/CompatibilityTest.cpp
        impl/config/CryConfigCreatorTest.cpp
        impl/config/CryConfigFileTest.cpp
        impl/config/CryConfigTest.cpp
        impl/config/CryCipherTest.cpp
        impl/config/CryConfigLoaderTest.cpp
        impl/config/CryConfigConsoleTest.cpp
        impl/config/CryPasswordBasedKeyProviderTest.cpp
        impl/config/CryPresetPasswordBasedKeyProviderTest.cpp
        impl/filesystem/CryFsTest.cpp
        impl/filesystem/CryNodeTest.cpp
        impl/filesystem/FileSystemTest.cpp
        impl/localstate/LocalStateMetadataTest.cpp
        impl/localstate/BasedirMetadataTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} my-gtest-main googletest cryfs)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
