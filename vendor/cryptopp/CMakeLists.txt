project(mycryptopp)

add_library(cryptopp dummy.cpp)
# note: include directory is called vendor_cryptopp instead of cryptopp to avoid include clashes with system headers
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(cryptopp PUBLIC -DCRYPTOPP_DEBUG)
endif()

if(NOT DISABLE_OPENMP)
    if((APPLE AND ((CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang") OR (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")))
        AND ((CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL "7.0") AND (CMAKE_VERSION VERSION_LESS "3.12.0")))
        # Workaround because older cmake on apple doesn't support FindOpenMP
        message(STATUS "Applying workaround for OSX OpenMP")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Xclang -fopenmp")
        target_link_libraries(cryptopp PRIVATE -lomp)
    else()
        find_package(OpenMP)
        if (NOT OpenMP_CXX_LIBRARIES)
          # Older CMake didn't have OpenMP_CXX_LIBRARIES defined yet, but passed the library in OpenMP_CXX_FLAGS
          set(OpenMP_CXX_LIBRARIES ${OpenMP_CXX_FLAGS})
        endif()
        if(OPENMP_FOUND)
            message(STATUS "Using OpenMP flags: ${OpenMP_CXX_FLAGS}")
            message(STATUS "Using OpenMP library: ${OpenMP_CXX_LIBRARIES}")
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
            target_link_libraries(cryptopp PRIVATE ${OpenMP_CXX_LIBRARIES})
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
