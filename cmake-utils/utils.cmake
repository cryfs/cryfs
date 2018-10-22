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
            target_compile_options(${TARGET} PRIVATE -std=c++14)
        else(COMPILER_HAS_CPP14_SUPPORT)
            check_cxx_compiler_flag("-std=c++1y" COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
            if (COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
                target_compile_options(${TARGET} PRIVATE -std=c++1y)
            else()
                message(FATAL_ERROR "Compiler doesn't support C++14")
            endif()
        endif(COMPILER_HAS_CPP14_SUPPORT)
    endif("${CMAKE_VERSION}" VERSION_GREATER "3.1")
    # Ideally, we'd like to use libc++ on linux as well, but:
    #    - http://stackoverflow.com/questions/37096062/get-a-basic-c-program-to-compile-using-clang-on-ubuntu-16
    #    - https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=808086
    # so only use it on Apple systems...
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang" AND APPLE)
        target_compile_options(${TARGET} PUBLIC -stdlib=libc++)
    endif(CMAKE_CXX_COMPILER_ID MATCHES "Clang" AND APPLE)
endfunction(target_activate_cpp14)

# Find clang-tidy executable (for use in target_enable_style_warnings)
if (USE_CLANG_TIDY)
    find_program(
      CLANG_TIDY_EXE
      NAMES "clang-tidy"
      DOC "Path to clang-tidy executable"
    )
    if(NOT CLANG_TIDY_EXE)
      message(FATAL_ERROR "clang-tidy not found. Please install clang-tidy or run without -DUSE_CLANG_TIDY=on.")
    else()
      set(CLANG_TIDY_OPTIONS "-system-headers=0")
      if (CLANG_TIDY_WARNINGS_AS_ERRORS)
          set(CLANG_TIDY_OPTIONS "${CLANG_TIDY_OPTIONS}" "-warnings-as-errors=*")
      endif()
      message(STATUS "Clang-tidy is enabled. Executable: ${CLANG_TIDY_EXE} Arguments: ${CLANG_TIDY_OPTIONS}")
      set(CLANG_TIDY_CLI "${CLANG_TIDY_EXE}" "${CLANG_TIDY_OPTIONS}")
    endif()
endif()

# Find iwyu (for use in target_enable_style_warnings)
if (USE_IWYU)
    find_program(
      IWYU_EXE NAMES
      include-what-you-use
      iwyu
    )
    if(NOT IWYU_EXE)
        message(FATAL_ERROR "include-what-you-use not found. Please install iwyu or run without -DUSE_IWYU=on.")
    else()
        message(STATUS "iwyu found: ${IWYU_EXE}")
        set(DO_IWYU "${IWYU_EXE}")
    endif()
endif()

#################################################
# Enable style compiler warnings
#
#  Uses: target_enable_style_warnings(buildtarget)
#################################################
function(target_enable_style_warnings TARGET)
    if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "MSVC")
        # TODO
    elseif ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "Clang" OR "${CMAKE_CXX_COMPILER_ID}" STREQUAL "AppleClang")
        target_compile_options(${TARGET} PRIVATE -Wall -Wextra -Wold-style-cast -Wcast-align -Wno-unused-command-line-argument) # TODO consider -Wpedantic -Wchkp -Wcast-qual -Wctor-dtor-privacy -Wdisabled-optimization -Wformat=2 -Winit-self -Wlogical-op -Wmissing-include-dirs -Wnoexcept -Wold-style-cast -Woverloaded-virtual -Wredundant-decls -Wshadow -Wsign-promo -Wstrict-null-sentinel -Wstrict-overflow=5 -Wundef -Wno-unused -Wno-variadic-macros -Wno-parentheses -fdiagnostics-show-option -Wconversion and others?
    elseif ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "GNU")
        target_compile_options(${TARGET} PRIVATE -Wall -Wextra -Wold-style-cast -Wcast-align -Wno-maybe-uninitialized) # TODO consider -Wpedantic -Wchkp -Wcast-qual -Wctor-dtor-privacy -Wdisabled-optimization -Wformat=2 -Winit-self -Wlogical-op -Wmissing-include-dirs -Wnoexcept -Wold-style-cast -Woverloaded-virtual -Wredundant-decls -Wshadow -Wsign-promo -Wstrict-null-sentinel -Wstrict-overflow=5 -Wundef -Wno-unused -Wno-variadic-macros -Wno-parentheses -fdiagnostics-show-option -Wconversion and others?
    endif()

    if (USE_WERROR)
        target_compile_options(${TARGET} PRIVATE -Werror)
    endif()

    # Enable clang-tidy
    if(USE_CLANG_TIDY)
        set_target_properties(
          ${TARGET} PROPERTIES
          CXX_CLANG_TIDY "${CLANG_TIDY_CLI}"
        )
    endif()
    if(USE_IWYU)
        set_target_properties(
          ${TARGET} PROPERTIES
          CXX_INCLUDE_WHAT_YOU_USE "${DO_IWYU}"
        )
    endif()
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
    set(BOOST_THREAD_VERSION 4)
    find_package(Boost 1.56.0
            REQUIRED
            COMPONENTS ${ARGN})
    target_include_directories(${TARGET} SYSTEM PUBLIC ${Boost_INCLUDE_DIRS})
    target_link_libraries(${TARGET} PUBLIC ${Boost_LIBRARIES})
    target_compile_definitions(${TARGET} PUBLIC BOOST_THREAD_VERSION=4)
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

include(cmake-utils/TargetArch.cmake)
function(get_target_architecture output_var)
	target_architecture(local_output_var)
	set(${output_var} ${local_output_var} PARENT_SCOPE)
endfunction()
