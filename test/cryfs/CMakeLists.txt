project (cryfs-test)

set(SOURCES
    cli/CallAfterTimeoutTest.cpp
    cli/testutils/CliTest.cpp
    cli/CliTest_Setup.cpp
    cli/CliTest_WrongEnvironment.cpp
    cli/program_options/UtilsTest.cpp
    cli/program_options/ProgramOptionsTest.cpp
    cli/program_options/ParserTest.cpp
    cli/CliTest_ShowingHelp.cpp
    cli/VersionCheckerTest.cpp
    cli/VersionCompareTest.cpp
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
    filesystem/CryFsTest.cpp
    filesystem/FileSystemTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} googletest cryfs_lib)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
