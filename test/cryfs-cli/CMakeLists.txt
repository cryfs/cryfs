project (cryfs-cli-test)

set(SOURCES
    CallAfterTimeoutTest.cpp
    testutils/CliTest.cpp
    CliTest_Setup.cpp
    CliTest_WrongEnvironment.cpp
    program_options/UtilsTest.cpp
    program_options/ProgramOptionsTest.cpp
    program_options/ParserTest.cpp
    CliTest_ShowingHelp.cpp
    EnvironmentTest.cpp
    VersionCheckerTest.cpp
    CliTest_IntegrityCheck.cpp
    CryfsUnmountTest.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} googletest cryfs-cli cryfs-unmount fspp-fuse)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
