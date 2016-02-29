project (cryfs-lib-usage-test)

set(SOURCES
        Load_Test_With_Filesystem.cpp
        Load_Test_Without_Filesystem.cpp
        Mount_Test.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} googletest cryfs cpp-utils)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
target_git_version_init(${PROJECT_NAME})
