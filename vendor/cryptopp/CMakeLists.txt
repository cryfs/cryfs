project(mycryptopp)

add_library(cryptopp dummy.cpp)
# note: include directory is called vendor_cryptopp instead of cryptopp to avoid include clashes with system headers
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

# Forward debug build info (i.e. set CRYPTOPP_DEBUG variable if building in debug mode)
target_compile_definitions(cryptopp PUBLIC $<$<CONFIG:Debug>:CRYPTOPP_DEBUG>) # add to all targets depending on this
add_compile_options($<$<CONFIG:Debug>:-DCRYPTOPP_DEBUG>) # add to stuff built in subdirectories (like the actual library)

if(NOT DISABLE_OPENMP)
    if (MSVC)
        message(WARNING "MSVC does not support the OpenMP 4.0 standard used by Crypto++. Disabling OpenMP. This can cause degraded performance.")
    else()
        find_package(OpenMP)

        if (OPENMP_FOUND OR OPENMP_CXX_FOUND)
            message(STATUS "Found libomp without any special flags")
        endif()

        # If OpenMP wasn't found, try if we can find it in the default Macports location
        if((NOT OPENMP_FOUND) AND (NOT OPENMP_CXX_FOUND) AND EXISTS "/opt/local/lib/libomp/libomp.dylib") # older cmake uses OPENMP_FOUND, newer cmake also sets OPENMP_CXX_FOUND, homebrew installations seem only to get the latter set.
            set(OpenMP_CXX_FLAGS "-Xpreprocessor -fopenmp -I/opt/local/include/libomp/")
            set(OpenMP_CXX_LIB_NAMES omp)
            set(OpenMP_omp_LIBRARY /opt/local/lib/libomp/libomp.dylib)

            find_package(OpenMP)
            if (OPENMP_FOUND OR OPENMP_CXX_FOUND)
                message(STATUS "Found libomp in macports default location.")
            else()
                message(FATAL_ERROR "Didn't find libomp. Tried macports default location but also didn't find it.")
            endif()
        endif()

        # If OpenMP wasn't found, try if we can find it in the default Homebrew location
        if((NOT OPENMP_FOUND) AND (NOT OPENMP_CXX_FOUND) AND EXISTS "/usr/local/opt/libomp/lib/libomp.dylib")
            set(OpenMP_CXX_FLAGS "-Xpreprocessor -fopenmp -I/usr/local/opt/libomp/include")
            set(OpenMP_CXX_LIB_NAMES omp)
            set(OpenMP_omp_LIBRARY /usr/local/opt/libomp/lib/libomp.dylib)

            find_package(OpenMP)
            if (OPENMP_FOUND OR OPENMP_CXX_FOUND)
                message(STATUS "Found libomp in homebrew default location.")
            else()
                message(FATAL_ERROR "Didn't find libomp. Tried homebrew default location but also didn't find it.")
            endif()
        endif()

        set(Additional_OpenMP_Libraries_Workaround "")

        # Workaround because older cmake on apple doesn't support FindOpenMP
        if((NOT OPENMP_FOUND) AND (NOT OPENMP_CXX_FOUND))
            if((APPLE AND ((CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang") OR (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")))
                    AND ((CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL "7.0") AND (CMAKE_VERSION VERSION_LESS "3.12.0")))
                message(STATUS "Applying workaround for OSX OpenMP with old cmake that doesn't have FindOpenMP")
                set(OpenMP_CXX_FLAGS "-Xclang -fopenmp")
                set(Additional_OpenMP_Libraries_Workaround "-lomp")
            else()
                message(FATAL_ERROR "Did not find OpenMP. Build with -DDISABLE_OPENMP=ON if you want to allow this and are willing to take the performance hit.")
            endif()
        endif()

        if(NOT TARGET OpenMP::OpenMP_CXX)
            # We're on cmake < 3.9, handle behavior of the old FindOpenMP implementation
            message(STATUS "Applying workaround for old CMake that doesn't define FindOpenMP using targets")
            add_library(OpenMP_TARGET INTERFACE)
            add_library(OpenMP::OpenMP_CXX ALIAS OpenMP_TARGET)
            target_compile_options(OpenMP_TARGET INTERFACE ${OpenMP_CXX_FLAGS}) # add to all targets depending on this
            find_package(Threads REQUIRED)
            target_link_libraries(OpenMP_TARGET INTERFACE Threads::Threads)
            target_link_libraries(OpenMP_TARGET INTERFACE ${OpenMP_CXX_FLAGS} ${Additional_OpenMP_Libraries_Workaround})
        endif()

        target_link_libraries(cryptopp PUBLIC ${OpenMP_CXX_FLAGS}) # Workaround for Ubuntu 18.04 that otherwise doesn't set -fopenmp for linking
        target_link_libraries(cryptopp PUBLIC OpenMP::OpenMP_CXX)

        # also add these flags to the third party Crypto++ build setup that is built in a subdirectory
        message(STATUS "OpenMP flags: ${OpenMP_CXX_FLAGS}")
        string(REPLACE " " ";" REPLACED_FLAGS ${OpenMP_CXX_FLAGS})
        add_compile_options(${REPLACED_FLAGS})
    endif()
else()
    message(WARNING "OpenMP is disabled. This can cause degraded performance.")
endif()


set(BUILD_TESTING OFF CACHE BOOL "")
set(BUILD_DOCUMENTATION OFF CACHE BOOL "")
set(BUILD_SHARED OFF CACHE BOOL "")
set(BUILD_STATIC ON CACHE BOOL "")
add_subdirectory(vendor_cryptopp EXCLUDE_FROM_ALL)

target_link_libraries(cryptopp PRIVATE cryptopp-static)
