project (cryfs-test)

set(SOURCES
    config/crypto/CryConfigEncryptorFactoryTest.cpp
    config/crypto/outer/OuterConfigTest.cpp
    config/crypto/outer/OuterEncryptorTest.cpp
    config/crypto/inner/ConcreteInnerEncryptorTest.cpp
    config/crypto/inner/InnerConfigTest.cpp
    config/crypto/CryConfigEncryptorTest.cpp
    config/CompatibilityTest.cpp
    config/CryConfigCreatorTest.cpp
    config/CryConfigFileTest.cpp
    config/CryConfigTest.cpp
    config/CryCipherTest.cpp
    config/CryConfigLoaderTest.cpp
    config/CryConfigConsoleTest.cpp
    config/CryPasswordBasedKeyProviderTest.cpp
    config/CryPresetPasswordBasedKeyProviderTest.cpp
    filesystem/CryFsTest.cpp
    filesystem/CryNodeTest.cpp
    filesystem/FileSystemTest.cpp
    localstate/LocalStateMetadataTest.cpp
    localstate/BasedirMetadataTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} googletest cryfs)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
