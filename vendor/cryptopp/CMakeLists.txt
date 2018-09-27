project(mycryptopp)

add_library(cryptopp dummy.cpp)
# note: include directory is called vendor_cryptopp instead of cryptopp to avoid include clashes with system headers
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

# Forward debug build info (i.e. set CRYPTOPP_DEBUG variable if building in debug mode)
target_compile_definitions(cryptopp PUBLIC $<$<CONFIG:Debug>:CRYPTOPP_DEBUG>) # add to all targets depending on this
add_compile_options($<$<CONFIG:Debug>:-DCRYPTOPP_DEBUG>) # add to stuff built in subdirectories (like the actual library)

if(NOT DISABLE_OPENMP)
    if((APPLE AND ((CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang") OR (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")))
        AND ((CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL "7.0") AND (CMAKE_VERSION VERSION_LESS "3.12.0")))
        # Workaround because older cmake on apple doesn't support FindOpenMP
        message(STATUS "Applying workaround for OSX OpenMP")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Xclang -fopenmp")
        target_compile_options(cryptopp PUBLIC -Xclang -fopenmp) # also add these flags to all libraries depending on this
        target_link_libraries(cryptopp PRIVATE -lomp)
    elseif(MSVC)
        message(WARNING "MSVC does not support the OpenMP 4.0 standard used by Crypto++. Disabling OpenMP. This can cause degraded performance.")
    else()
        find_package(OpenMP)
        if (NOT OpenMP_CXX_LIBRARIES)
          # Older CMake didn't have OpenMP_CXX_LIBRARIES defined yet, but passed the library in OpenMP_CXX_FLAGS
          message(STATUS "We are on an older CMake that does not set OpenMP_CXX_LIBRARIES. Setting it from OpenMP_CXX_FLAGS.")
          set(OpenMP_CXX_LIBRARIES ${OpenMP_CXX_FLAGS})
        endif()
        if(OPENMP_FOUND)
            message(STATUS "Using OpenMP flags: ${OpenMP_CXX_FLAGS}")
            message(STATUS "Using OpenMP library: ${OpenMP_CXX_LIBRARIES}")
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
            target_link_libraries(cryptopp PRIVATE ${OpenMP_CXX_LIBRARIES})
            # also add these flags to all libraries depending on this
            string(REPLACE " " ";" REPLACED_FLAGS ${OpenMP_CXX_FLAGS})
            target_compile_options(cryptopp PUBLIC ${REPLACED_FLAGS})
        else(OPENMP_FOUND)
            message(FATAL_ERROR "Did not find OpenMP. Build with -DDISABLE_OPENMP=ON if you want to allow this and are willing to take the performance hit.")
        endif(OPENMP_FOUND)
    endif()
else(NOT DISABLE_OPENMP)
    message(WARNING "OpenMP is disabled. This can cause degraded performance.")
endif(NOT DISABLE_OPENMP)


set(BUILD_TESTING OFF)
set(BUILD_DOCUMENTATION OFF)
set(BUILD_SHARED OFF)
set(BUILD_STATIC ON)
add_subdirectory(vendor_cryptopp EXCLUDE_FROM_ALL)

target_link_libraries(cryptopp PRIVATE cryptopp-static)
