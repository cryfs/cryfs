include(CheckCXXCompilerFlag)

###################################################
#  Activate C++14
#
#  Uses: target_activate_cpp14(buildtarget)
###################################################
function(target_activate_cpp14 TARGET)
    if("${CMAKE_VERSION}" VERSION_GREATER "3.1")
        set_property(TARGET ${TARGET} PROPERTY CXX_STANDARD 14)
        set_property(TARGET ${TARGET} PROPERTY CXX_STANDARD_REQUIRED ON)
    else("${CMAKE_VERSION}" VERSION_GREATER "3.1")
        check_cxx_compiler_flag("-std=c++14" COMPILER_HAS_CPP14_SUPPORT)
        if (COMPILER_HAS_CPP14_SUPPORT)
            target_compile_options(${TARGET} PUBLIC -std=c++14)
        else(COMPILER_HAS_CPP14_SUPPORT)
            check_cxx_compiler_flag("-std=c++1y" COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
            if (COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
                target_compile_options(${TARGET} PUBLIC -std=c++1y)
            else()
                message(FATAL_ERROR "Compiler doesn't support C++14")
            endif()
        endif(COMPILER_HAS_CPP14_SUPPORT)
    endif("${CMAKE_VERSION}" VERSION_GREATER "3.1")
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        target_compile_options(${TARGET} PUBLIC -stdlib=libc++)
    endif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
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
    if(NOT DEFINED Boost_USE_STATIC_LIBS OR Boost_USE_STATIC_LIBS)
        # Many supported systems don't have boost >= 1.56. Better link it statically.
        message(STATUS "Boost will be statically linked")
        set(Boost_USE_STATIC_LIBS ON)
    else(NOT DEFINED Boost_USE_STATIC_LIBS OR Boost_USE_STATIC_LIBS)
        message(STATUS "Boost will be dynamically linked")
        set(Boost_USE_STATIC_LIBS OFF)
    endif(NOT DEFINED Boost_USE_STATIC_LIBS OR Boost_USE_STATIC_LIBS)
    find_package(Boost 1.56.0
            REQUIRED
            COMPONENTS ${ARGN})
    target_include_directories(${TARGET} SYSTEM PUBLIC ${Boost_INCLUDE_DIRS})
    target_link_libraries(${TARGET} PUBLIC ${Boost_LIBRARIES})
    if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
      # Also link to rt, because boost thread needs that.
      target_link_libraries(${TARGET} PUBLIC rt)
    endif(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
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

##################################################
# Specify that a specific minimal version of clang is required
#
# Uses:
#  require_clang_version(3.5)
##################################################
function(require_clang_version VERSION)
    if (CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        if (CMAKE_CXX_COMPILER_VERSION VERSION_LESS ${VERSION})
            message(FATAL_ERROR "Needs at least clang version ${VERSION}, found clang ${CMAKE_CXX_COMPILER_VERSION}")
        endif (CMAKE_CXX_COMPILER_VERSION VERSION_LESS ${VERSION})
    endif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
endfunction(require_clang_version)

##################################################
# Find the location of a library and return its full path in OUTPUT_VARIABLE.
# If PATH_VARIABLE points to a defined variable, then the library will only be searched in this path.
# If PATH_VARIABLE points to a undefined variable, default system locations will be searched.
#
# Uses (the following will search for fuse in system locations by default, and if the user passes -DFUSE_LIB_PATH to cmake, it will only search in this path.
#  find_library_with_path(MYLIBRARY fuse FUSE_LIB_PATH)
#  target_link_library(target ${MYLIBRARY})
##################################################
function(find_library_with_path OUTPUT_VARIABLE LIBRARY_NAME PATH_VARIABLE)
    if(${PATH_VARIABLE})
        find_library(${OUTPUT_VARIABLE} ${LIBRARY_NAME} PATHS ${${PATH_VARIABLE}} NO_DEFAULT_PATH)
        if (${OUTPUT_VARIABLE} MATCHES NOTFOUND)
            message(FATAL_ERROR "Didn't find ${LIBRARY_NAME} in path specified by the ${PATH_VARIABLE} parameter (${${PATH_VARIABLE}}). Pass in the correct path or remove the parameter to try common system locations.")
        else(${OUTPUT_VARIABLE} MATCHES NOTFOUND)
            message(STATUS "Found ${LIBRARY_NAME} in user-defined path ${${PATH_VARIABLE}}")
        endif(${OUTPUT_VARIABLE} MATCHES NOTFOUND)
    else(${PATH_VARIABLE})
        find_library(${OUTPUT_VARIABLE} ${LIBRARY_NAME})
        if (${OUTPUT_VARIABLE} MATCHES NOTFOUND)
            message(FATAL_ERROR "Didn't find ${LIBRARY_NAME} library. If ${LIBRARY_NAME} is installed, try passing in the library location with -D${PATH_VARIABLE}=/path/to/${LIBRARY_NAME}/lib.")
        else(${OUTPUT_VARIABLE} MATCHES NOTFOUND)
            message(STATUS "Found ${LIBRARY_NAME} in system location")
        endif(${OUTPUT_VARIABLE} MATCHES NOTFOUND)
    endif(${PATH_VARIABLE})
endfunction(find_library_with_path)
