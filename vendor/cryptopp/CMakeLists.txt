project(mycryptopp)

add_library(cryptopp dummy.cpp)
target_include_directories(cryptopp SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

find_package(OpenMP)
if(OPENMP_FOUND)
    message(STATUS "Building crypto++ with OpenMP")
    set(CMAKE_CXX_FLAGS ${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS})
    target_link_libraries(cryptopp PRIVATE ${OpenMP_CXX_FLAGS})
else(OPENMP_FOUND)
    message(WARN "Did not find OpenMP. Performance might be degraded.")
endif(OPENMP_FOUND)


set(BUILD_TESTING OFF)
set(BUILD_DOCUMENTATION OFF)
set(BUILD_SHARED OFF)
set(BUILD_STATIC ON)
add_subdirectory(cryptopp EXCLUDE_FROM_ALL)

target_link_libraries(cryptopp PRIVATE cryptopp-static)

