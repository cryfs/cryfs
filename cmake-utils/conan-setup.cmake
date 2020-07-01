macro(setup_conan)
    include(cmake-utils/conan.cmake)

    if(MSVC)
        conan_cmake_run(
            CONANFILE conanfile.py
            BUILD missing)
    else()
        conan_cmake_run(
            CONANFILE conanfile.py
            # We'd like to use "BUILD missing" but that doesn't work because conan sometimes seems to download prebuilt packages with compiler.libcxx=libstdc++ even though we specify compiler.libcxx=libstdc++11.
            # see https://github.com/cryfs/cryfs/issues/336 and https://github.com/conan-io/conan/issues/7264
            BUILD all
            SETTINGS compiler.libcxx=libstdc++11)
    endif()
    conan_basic_setup(TARGETS SKIP_STD)

    if(CONAN_SETTINGS_COMPILER_LIBCXX STREQUAL "libstdc++")
        # TODO Test this warning works correctly and that the proposed solution in the warning message works.
        message(FATAL_ERROR "Conan is set up to build against libstdc++ (i.e. the legacy GCC ABI). We only support libstdc++11 (i.e. the new GCC ABI).\nPlease add the '-s compiler.libcxx=libstdc++11' argument when running 'conan install'.")
    endif()
endmacro()
