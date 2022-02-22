project(mycryptopp)

add_library(cryptopp INTERFACE)
# note: include directory is called vendor_cryptopp instead of cryptopp to avoid include clashes with system headers
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

# Forward debug build info (i.e. set CRYPTOPP_DEBUG variable if building in debug mode)
target_compile_definitions(cryptopp INTERFACE $<$<CONFIG:Debug>:CRYPTOPP_DEBUG>) # add to all targets depending on this
add_compile_options($<$<CONFIG:Debug>:-DCRYPTOPP_DEBUG>) # add to stuff built in subdirectories (like the actual library)

if (NOT DISABLE_OPENMP)
    set(USE_OPENMP ON CACHE BOOL "")
endif()
set(CRYPTOPP_BUILD_TESTING OFF CACHE BOOL "")
set(BUILD_DOCUMENTATION OFF CACHE BOOL "")
set(BUILD_SHARED OFF CACHE BOOL "")
set(BUILD_STATIC ON CACHE BOOL "")
set(USE_INTERMEDIATE_OBJECTS_TARGET OFF CACHE BOOL "")
set(cryptocpp_DISPLAY_CMAKE_SUPPORT_WARNING OFF CACHE BOOL "")
add_subdirectory(vendor_cryptopp EXCLUDE_FROM_ALL)

target_link_libraries(cryptopp INTERFACE cryptopp-static)
