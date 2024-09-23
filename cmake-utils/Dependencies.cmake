# Setup range-v3 dependency
find_package(range-v3 REQUIRED)
add_library(CryfsDependencies_range-v3 INTERFACE)
target_link_libraries(CryfsDependencies_range-v3 INTERFACE range-v3::range-v3)

# Setup boost dependency
set(Boost_USE_STATIC_LIBS OFF)
find_package(Boost 1.84.0
        REQUIRED
        COMPONENTS filesystem system thread chrono program_options)
add_library(CryfsDependencies_boost INTERFACE)
target_link_libraries(CryfsDependencies_boost INTERFACE Boost::boost Boost::filesystem Boost::thread Boost::chrono Boost::program_options)
if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
    # Also link to rt, because boost thread needs that.
    target_link_libraries(CryfsDependencies_boost INTERFACE rt)
endif()

# Setup spdlog dependency
find_package(spdlog REQUIRED)
add_library(CryfsDependencies_spdlog INTERFACE)
target_link_libraries(CryfsDependencies_spdlog INTERFACE spdlog::spdlog)

# Setup libcurl dependency
find_package(CURL REQUIRED)
add_library(CryfsDependencies_libcurl INTERFACE)
target_link_libraries(CryfsDependencies_libcurl INTERFACE CURL::libcurl)

# Setup gtest dependency
if (BUILD_TESTING)
    find_package(GTest REQUIRED)
    add_library(CryfsDependencies_gtest INTERFACE)
    target_link_libraries(CryfsDependencies_gtest INTERFACE GTest::gtest GTest::gmock)
endif()