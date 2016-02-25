project (cryfs-lib-usage-test C)

add_executable(${PROJECT_NAME} init_and_free.c)
target_link_libraries(${PROJECT_NAME} cryfs)
