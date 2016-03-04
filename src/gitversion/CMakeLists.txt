project (gitversion)

include(gitversion.cmake)
get_git_version(GIT_VERSION)

add_library(${PROJECT_NAME} STATIC gitversion.cpp versionstring.cpp parser.cpp)
target_link_libraries(${PROJECT_NAME})
target_compile_definitions(${PROJECT_NAME} PRIVATE GIT_VERSION_STRING="${GIT_VERSION}")
target_add_boost(${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

# Needed so it can be linked in to the shared cryfs library
set_target_properties(${PROJECT_NAME} PROPERTIES POSITION_INDEPENDENT_CODE on)
