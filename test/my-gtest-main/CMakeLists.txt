project (my-gtest-main)

set(SOURCES
    my-gtest-main.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})
target_link_libraries(${PROJECT_NAME} PUBLIC CryfsDependencies_gtest cpp-utils)
target_add_boost(${PROJECT_NAME} filesystem system)
target_include_directories(${PROJECT_NAME} PUBLIC .)

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp20(${PROJECT_NAME})
