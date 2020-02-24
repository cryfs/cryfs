macro(setup_conan)
    include(cmake-utils/conan.cmake)

    conan_cmake_run(
        CONANFILE conanfile.py
        BUILD missing)
    conan_basic_setup(TARGETS SKIP_STD)

    if(CONAN_SETTINGS_COMPILER_LIBCXX STREQUAL "libstdc++")
        # TODO Test this warning works correctly and that the proposed solution in the warning message works.
        message(FATAL_ERROR "Conan is set up to build against libstdc++ (i.e. the legacy GCC ABI). We only support libstdc++11 (i.e. the new GCC ABI).\nPlease add the '-s compiler.libcxx=libstdc++11' argument when running 'conan install'.")
    endif()
endmacro()
