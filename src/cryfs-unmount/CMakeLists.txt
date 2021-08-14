project (cryfs-unmount)
INCLUDE(GNUInstallDirs)

set(SOURCES
	program_options/ProgramOptions.cpp
	program_options/Parser.cpp
	Cli.cpp
)

add_library(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils cryfs fspp-fuse)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
target_add_boost(${PROJECT_NAME})

add_executable(${PROJECT_NAME}_bin main_unmount.cpp)
set_target_properties(${PROJECT_NAME}_bin PROPERTIES OUTPUT_NAME cryfs-unmount)
target_link_libraries(${PROJECT_NAME}_bin PUBLIC ${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME}_bin)
target_activate_cpp14(${PROJECT_NAME}_bin)

install(TARGETS ${PROJECT_NAME}_bin
        DESTINATION ${CMAKE_INSTALL_BINDIR}
)
