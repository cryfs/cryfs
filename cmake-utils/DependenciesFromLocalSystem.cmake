# This configuration file can be used to build CryFS against local dependencies instead of using Conan.
#
# Example:
# $ mkdir build && cd build && cmake .. -DDEPENDENCY_CONFIG=../cmake-utils/DependenciesFromLocalSystem.cmake
#
# Note that this is only provided as an example and not officially supported. Please still open issues
# on GitHub if it doesn't work though.
#
# There's another file in this directory, DependenciesFromConan.cmake, which, well, gets the dependencies from
# Conan instead of from the local system. This is the default. You can also create your own file to tell the build
# how to get its dependencies, for example you can mix and match, get some dependencies from Conan and others
# from the local system. If you mix and match Conan and local dependencies, please call conan_basic_setup()
# **after** running all find_package() for your local dependencies, otherwise find_package() might also find
# the versions from Conan.
#
# Note that if you use dependencies from the local system, you're very likely using different versions of the
# dependencies than were used in the development of CryFS. The official version of each dependency required is
# listed in conanfile.py. Different versions might work but are untested. Please intensively test your CryFS build
# if you build it with different versions of the dependencies.


function(check_target_is_not_from_conan TARGET)
    get_target_property(INCLUDE_DIRS ${TARGET} INTERFACE_INCLUDE_DIRECTORIES)
    if("${INCLUDE_DIRS}" MATCHES "conan")
        message(WARNING "It seems setting up the local ${TARGET} dependency didn't work correctly and it got the version from Conan instead. Please set up cmake so that it sets up conan after all local dependencies are defined.")
    endif()
endfunction()




# Setup range-v3 dependency
find_package(range-v3 REQUIRED)
check_target_is_not_from_conan(range-v3::range-v3)
add_library(CryfsDependencies_range-v3 INTERFACE)
target_link_libraries(CryfsDependencies_range-v3 INTERFACE range-v3::range-v3)




# Setup boost dependency
set(Boost_USE_STATIC_LIBS OFF)
find_package(Boost 1.65.1
        REQUIRED
        COMPONENTS filesystem system thread chrono program_options)
check_target_is_not_from_conan(Boost::boost)
add_library(CryfsDependencies_boost INTERFACE)
target_link_libraries(CryfsDependencies_boost INTERFACE Boost::boost Boost::filesystem Boost::thread Boost::chrono Boost::program_options)
if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
    # Also link to rt, because boost thread needs that.
    target_link_libraries(CryfsDependencies_boost INTERFACE rt)
endif()




# Setup spdlog dependency
find_package(spdlog REQUIRED)
check_target_is_not_from_conan(spdlog::spdlog)
add_library(CryfsDependencies_spdlog INTERFACE)
target_link_libraries(CryfsDependencies_spdlog INTERFACE spdlog::spdlog)
