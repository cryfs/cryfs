add_subdirectory(impl)

project(cryfs)

set(SOURCES
        cryfs.cpp
        lib/cryfs_load_context.cpp
        lib/cryfs_mount_handle.cpp
        lib/mount_handle_list.cpp
        lib/cryfs_unmounter.cpp
        lib/utils/CallAfterTimeout.cpp
        lib/utils/filesystem_checks.cpp
)

add_library(${PROJECT_NAME} SHARED ${SOURCES})
target_link_libraries(${PROJECT_NAME} PRIVATE cryfs-impl)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

# The library should only export the functions we declared to be exported
include(GenerateExportHeader)
add_compiler_export_flags(EXPORT_FLAGS)
separate_arguments(EXPORT_FLAGS)
target_compile_options(${PROJECT_NAME} PRIVATE ${EXPORT_FLAGS})
generate_export_header(${PROJECT_NAME})
target_include_directories(${PROJECT_NAME} PUBLIC ${PROJECT_BINARY_DIR})

install(TARGETS ${PROJECT_NAME}
        DESTINATION lib
        CONFIGURATIONS Release
)
install(FILES
        cryfs.h
        ${PROJECT_BINARY_DIR}/cryfs_export.h
        DESTINATION include/cryfs
)
