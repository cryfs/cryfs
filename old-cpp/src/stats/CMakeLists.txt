project (stats)

set(SOURCES
        main.cpp
        traversal.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} PUBLIC cryfs cpp-utils gitversion)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
set_target_properties(${PROJECT_NAME} PROPERTIES OUTPUT_NAME cryfs-stats)
