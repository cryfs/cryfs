include(CheckCXXCompilerFlag)

###################################################
#  Activate C++14
#
#  Uses: target_activate_cpp14(buildtarget)
###################################################
function(target_activate_cpp14 TARGET)
    check_cxx_compiler_flag("-std=c++14" COMPILER_HAS_CPP14_SUPPORT)
    IF (COMPILER_HAS_CPP14_SUPPORT)
        target_compile_options(${TARGET} PUBLIC -std=c++14)
    ELSE()
        check_cxx_compiler_flag("-std=c++1y" COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
        IF (COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
            target_compile_options(${TARGET} PUBLIC -std=c++1y)
        ELSE()
            message(FATAL_ERROR "Compiler doesn't support C++14")
        ENDIF()
    ENDIF()
    IF(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        target_compile_options(${TARGET} PUBLIC -stdlib=libc++)
    ENDIF()
endfunction(target_activate_cpp14)

#################################################
# Enable style compiler warnings
#
#  Uses: target_enable_style_warnings(buildtarget)
#################################################
function(target_enable_style_warnings TARGET)
    target_compile_options(${TARGET} PRIVATE -Wall -Wextra)
endfunction(target_enable_style_warnings)

##################################################
# Add boost to the project
#
# Uses:
#  target_add_boost(buildtarget) # if you're only using header-only boost libs
#  target_add_boost(buildtarget system filesystem) # list all libraries to link against in the dependencies
##################################################
function(target_add_boost TARGET)
    # Load boost libraries
    set(Boost_USE_STATIC_LIBS ON) # Many supported systems don't have boost >= 1.56. Better link it statically.
    find_package(Boost 1.56.0
            REQUIRED
            COMPONENTS ${ARGN})
    target_include_directories(${TARGET} SYSTEM PRIVATE ${Boost_INCLUDE_DIRS})
    target_link_libraries(${TARGET} PRIVATE ${Boost_LIBRARIES})
endfunction(target_add_boost)

##################################################
# Specify that a specific minimal version of gcc is required
#
# Uses:
#  require_gcc_version(4.9)
##################################################
function(require_gcc_version VERSION)
    if (CMAKE_COMPILER_IS_GNUCXX)
        execute_process(COMMAND ${CMAKE_CXX_COMPILER} -dumpversion OUTPUT_VARIABLE GCC_VERSION)
        if (GCC_VERSION VERSION_LESS ${VERSION})
            message(FATAL_ERROR "Needs at least gcc version ${VERSION}, found gcc ${GCC_VERSION}")
        endif (GCC_VERSION VERSION_LESS ${VERSION})
    endif (CMAKE_COMPILER_IS_GNUCXX)
endfunction(require_gcc_version)

function(require_clang_version VERSION)
    if (CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        if (CMAKE_CXX_COMPILER_VERSION VERSION_LESS ${VERSION})
            message(FATAL_ERROR "Needs at least clang version ${VERSION}, found clang ${CMAKE_CXX_COMPILER_VERSION}")
        endif (CMAKE_CXX_COMPILER_VERSION VERSION_LESS ${VERSION})
    endif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
endfunction(require_clang_version)