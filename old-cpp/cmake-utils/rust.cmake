############################################################################################################################################################
#  Create a rust companion library for a C++ library target.
#  
#  Code in the C++ library will be able to call
#  into the rust library and the other way round.
#
#  Uses:
#    target_add_rust_companion(
#      cryfs-cli                                  # the name of the library target to create a companion for
#      RUST_LIB_NAME "libcryfs_cli.a"             # name of the rust staticlib created from Cargo.toml
#      RUST_CRATE_NAME "cryfs-cli"                # name of the rust crate as specified in Cargo.toml
#      RUST_BRIDGES "src/lib.rs" "src/lib2.rs"    # rust files containing cxx::bridges (see rust cxx crate)
#      RUST_DIR "rust"                            # directory with the rust project. Must be the name of a subdirectory of the current directory.
#   )
############################################################################################################################################################
function(target_add_rust_companion TARGET_NAME)
    set(options)
    set(oneValueArgs RUST_LIB_NAME RUST_CRATE_NAME RUST_TARGET_NAME TARGET_NAME RUST_DIR)
    set(multiValueArgs RUST_BRIDGES)
    cmake_parse_arguments(PARSE_ARGV 1 ARGS "${options}" "${oneValueArgs}" "${multiValueArgs}")

    if (CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(CARGO_CMD cargo build)
        set(TARGET_DIR "debug")
    else ()
        # TODO RelWithDebInfo, MinSizeRel
        set(CARGO_CMD cargo build --release)
        set(TARGET_DIR "release")
    endif ()

    # Build the list of .cc files that are generated from the rust cxx::bridges
    string(REGEX REPLACE "([^;]+)" "${CMAKE_CURRENT_BINARY_DIR}/${ARGS_RUST_DIR}/cxxbridge/${ARGS_RUST_CRATE_NAME}/\\1.cc" RUST_BRIDGE_CPP_FILES "${ARGS_RUST_BRIDGES}")

    add_library("${TARGET_NAME}_rustbridgefiles" STATIC ${RUST_BRIDGE_CPP_FILES})
    add_library("${TARGET_NAME}_rustcompanion" INTERFACE)
    target_include_directories("${TARGET_NAME}_rustcompanion" INTERFACE ${CMAKE_CURRENT_BINARY_DIR}/${ARGS_RUST_DIR})
    target_include_directories("${TARGET_NAME}_rustbridgefiles" PUBLIC ${CMAKE_CURRENT_BINARY_DIR}/${ARGS_RUST_DIR})


    # Enable cross-language LTO if the target wants lto
    get_target_property(TARGET_WANTS_LTO "${TARGET_NAME}" INTERPROCEDURAL_OPTIMIZATION)
    set(RUST_FLAGS "")
    if(TARGET_WANTS_LTO)
        if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "Clang" OR "${CMAKE_CXX_COMPILER_ID}" STREQUAL "AppleClang")
            string(REGEX MATCH "^[0-9]+(\.|$)" _CMAKE_CXX_COMPILER_VERSION_MAJOR "${CMAKE_CXX_COMPILER_VERSION}")
            # Rust 1.49 requires LLVM 9 or later, see https://github.com/rust-lang/rust/blob/master/RELEASES.md.
            # Also note LTO compatibility table at https://doc.rust-lang.org/rustc/linker-plugin-lto.html
            if ("${_CMAKE_CXX_COMPILER_VERSION_MAJOR}" GREATER_EQUAL 9)
                message(STATUS "Cross-language LTO enabled, using clang ${_CMAKE_CXX_COMPILER_VERSION_MAJOR}")
                set_property(TARGET "${TARGET_NAME}_rustcompanion" PROPERTY CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
                target_link_libraries(${TARGET_NAME} PUBLIC "-fuse-ld=lld")
                target_link_libraries("${TARGET_NAME}_rustcompanion" INTERFACE "-fuse-ld=lld")
                set(RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang" "-Clink-arg=-fuse-ld=lld")
            else()
                message(WARNING "Cross-language LTO for ${TARGET_NAME} disabled because we're on clang version ${_CMAKE_CXX_COMPILER_VERSION_MAJOR} which is too old")
            endif()
        else()
            message(WARNING "Cross-language LTO for ${TARGET_NAME} disabled because we're not using the clang compiler.")
        endif()
    else()
        message(STATUS "Cross-language LTO for ${TARGET_NAME} disabled because the target property INTERPROCEDURAL_OPTIMIZATION is not set on the ${TARGET_NAME} target")
    endif()

    file(GLOB_RECURSE RUST_SOURCE_FILES
        "${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_RUST_DIR}/*"
    )
    list(FILTER RUST_SOURCE_FILES EXCLUDE REGEX "/target/")
    add_custom_command(
        OUTPUT ${RUST_BRIDGE_CPP_FILES}
        COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}/${ARGS_RUST_DIR} RUSTFLAGS="${RUST_FLAGS}" ${CARGO_CMD}
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_RUST_DIR}
        DEPENDS ${RUST_SOURCE_FILES}
    )

    target_link_libraries("${TARGET_NAME}_rustcompanion" INTERFACE pthread dl)
    add_dependencies("${TARGET_NAME}_rustcompanion" "${TARGET_NAME}_rustbridgefiles")
    # There can be cyclic dependencies between rust bridge files and the compiled rust code. We need --start-group and --end-group for this.
    target_link_libraries("${TARGET_NAME}_rustcompanion" INTERFACE "-Wl,--start-group" "${CMAKE_CURRENT_BINARY_DIR}/lib${TARGET_NAME}_rustbridgefiles.a" "${CMAKE_CURRENT_BINARY_DIR}/${ARGS_RUST_DIR}/${TARGET_DIR}/${ARGS_RUST_LIB_NAME}" "-Wl,--end-group")

    target_link_libraries("${TARGET_NAME}" PUBLIC "${TARGET_NAME}_rustcompanion")

    add_test(NAME "${TARGET_NAME}_rustcompanion_test"
        COMMAND cargo test
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_RUST_DIR})
endfunction()
