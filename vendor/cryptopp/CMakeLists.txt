project(mycryptopp)

add_library(mycryptopp INTERFACE)
# note: include directory is called vendor_cryptopp instead of cryptopp to avoid include clashes with system headers
target_include_directories(mycryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

# Forward debug build info (i.e. set CRYPTOPP_DEBUG variable if building in debug mode)
target_compile_definitions(mycryptopp INTERFACE $<$<CONFIG:Debug>:CRYPTOPP_DEBUG>) # add to all targets depending on this
add_compile_options($<$<CONFIG:Debug>:-DCRYPTOPP_DEBUG>) # add to stuff built in subdirectories (like the actual library)

if (NOT DISABLE_OPENMP)
    set(CRYPTOPP_USE_OPENMP ON CACHE BOOL "")
endif()
set(CRYPTOPP_BUILD_TESTING OFF CACHE BOOL "")
set(CRYPTOPP_BUILD_DOCUMENTATION OFF CACHE BOOL "")
set(CRYPTOPP_BUILD_SHARED OFF CACHE BOOL "")
set(CRYPTOPP_USE_INTERMEDIATE_OBJECTS_TARGET OFF CACHE BOOL "")
set(CRYPTOPP_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/cryptopp-vendor")
add_subdirectory(cryptopp-cmake EXCLUDE_FROM_ALL)

target_link_libraries(mycryptopp INTERFACE cryptopp)
