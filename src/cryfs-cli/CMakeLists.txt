project (cryfs-cli)

set(SOURCES
        Cli.cpp
        VersionChecker.cpp
        CallAfterTimeout.cpp
        Environment.cpp
        program_options/utils.cpp
        program_options/ProgramOptions.cpp
        program_options/Parser.cpp
)

add_library(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} PUBLIC cryfs cpp-utils gitversion)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

add_executable(${PROJECT_NAME}_bin main.cpp)
set_target_properties(${PROJECT_NAME}_bin PROPERTIES OUTPUT_NAME cryfs)
target_link_libraries(${PROJECT_NAME}_bin PUBLIC ${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME}_bin)
target_activate_cpp14(${PROJECT_NAME}_bin)

install(TARGETS ${PROJECT_NAME}_bin
        DESTINATION bin
        CONFIGURATIONS Release
)
