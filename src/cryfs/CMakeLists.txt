add_subdirectory(impl)

project(cryfs)

add_library(${PROJECT_NAME} SHARED cryfs.cpp)
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
