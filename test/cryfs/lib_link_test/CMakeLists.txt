project (cryfs-lib-link-test C)

add_executable(${PROJECT_NAME} link_c_library.c)
target_link_libraries(${PROJECT_NAME} cryfs)

target_enable_style_warnings(${PROJECT_NAME})
