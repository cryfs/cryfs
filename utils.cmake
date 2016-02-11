include(CheckCXXCompilerFlag)

###################################################
#  Activate C++14
#
#  Uses: ACTIVATE_CPP14(buildtarget)
###################################################
function(ACTIVATE_CPP14 TARGET)
    CHECK_CXX_COMPILER_FLAG("-std=c++14" COMPILER_HAS_CPP14_SUPPORT)
    IF (COMPILER_HAS_CPP14_SUPPORT)
        TARGET_COMPILE_OPTIONS(${TARGET} PUBLIC -std=c++14)
    ELSE()
        CHECK_CXX_COMPILER_FLAG("-std=c++1y" COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
        IF (COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
            TARGET_COMPILE_OPTIONS(${TARGET} PUBLIC -std=c++1y)
        ELSE()
            MESSAGE(FATAL_ERROR "Compiler doesn't support C++14")
        ENDIF()
    ENDIF()
    IF(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        TARGET_COMPILE_OPTIONS(${TARGET} PUBLIC -stdlib=libc++)
    ENDIF()
endfunction(ACTIVATE_CPP14)

#################################################
# Enable style compiler warnings
#
#  Uses: ENABLE_STYLE_WARNINGS(buildtarget)
#################################################
function(ENABLE_STYLE_WARNINGS TARGET)
    TARGET_COMPILE_OPTIONS(${TARGET} PRIVATE -Wall -Wextra)
endfunction(ENABLE_STYLE_WARNINGS)

##################################################
# Add boost to the project
#
# Uses:
#  ADD_BOOST(buildtarget) # if you're only using header-only boost libs
#  ADD_BOOST(buildtarget system filesystem) # list all libraries to link against in the dependencies
##################################################
function(ADD_BOOST TARGET)
    # Load boost libraries
    find_package(Boost 1.56.0
            REQUIRED
            COMPONENTS ${ARGN})
    set(Boost_USE_STATIC_LIBS ON)
    target_include_directories(${TARGET} SYSTEM PRIVATE ${Boost_INCLUDE_DIRS})
    target_link_libraries(${TARGET} PRIVATE ${Boost_LIBRARIES})
endfunction()

##################################################
# Specify that a specific minimal version of gcc is required
#
# Uses:
#  REQUIRE_GCC_VERSION(4.9)
##################################################
function(REQUIRE_GCC_VERSION)
    if (CMAKE_COMPILER_IS_GNUCXX)
        execute_process(COMMAND ${CMAKE_CXX_COMPILER} -dumpversion OUTPUT_VARIABLE GCC_VERSION)
        if (GCC_VERSION VERSION_LESS ${ARGN})
            message(FATAL_ERROR "Needs at least gcc version ${ARGN}, found gcc ${GCC_VERSION}")
        endif (GCC_VERSION VERSION_LESS ${ARGN})
    endif (CMAKE_COMPILER_IS_GNUCXX)
endfunction()
