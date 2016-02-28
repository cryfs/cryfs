project (cryfs-lib-usage-test)

add_executable(${PROJECT_NAME} test_c_library.cpp)
target_link_libraries(${PROJECT_NAME} googletest cryfs cpp-utils)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
