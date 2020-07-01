macro(setup_conan)
    include(cmake-utils/conan.cmake)

    conan_cmake_run(
        CONANFILE conanfile.py
        BUILD missing)

    conan_basic_setup(TARGETS SKIP_STD)
endmacro()
