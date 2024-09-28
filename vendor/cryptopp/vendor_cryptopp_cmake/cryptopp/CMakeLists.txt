# ===-----------------------------------------------------------------------===#
# Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
# copy at https://opensource.org/licenses/BSD-3-Clause).
# SPDX-License-Identifier: BSD-3-Clause
# ===-----------------------------------------------------------------------===#

# Print useful information
message(STATUS "[cryptopp] CMake version ${CMAKE_VERSION}")
message(STATUS "[cryptopp] System ${CMAKE_SYSTEM_NAME}")
message(STATUS "[cryptopp] Processor ${CMAKE_SYSTEM_PROCESSOR}")

# Set the location where crypto++ has been setup if not passed to this script
# from the parent. Assume we are doing the manual build.
if(NOT DEFINED cryptopp_SOURCE_DIR)
    set(cryptopp_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR})
endif()

# Make RelWithDebInfo the default (it does e.g. add '-O2 -g -DNDEBUG' for GNU)
# If not in multi-configuration environments, no explicit build type or CXX
# flags are set by the user and if we are the root CMakeLists.txt file.
if(
    NOT CMAKE_CONFIGURATION_TYPES
    AND NOT CMAKE_NO_BUILD_TYPE
    AND NOT CMAKE_BUILD_TYPE
    AND NOT CMAKE_CXX_FLAGS
    AND CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR
)
    set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

include(CheckCXXCompilerFlag)

# We now carry around test programs. test_cxx.cpp is the default C++ one. Also
# see https://github.com/weidai11/cryptopp/issues/741.
set(TEST_PROG_DIR ${cryptopp_SOURCE_DIR}/TestPrograms)
set(TEST_CXX_FILE ${TEST_PROG_DIR}/test_cxx.cpp)

# https://github.com/noloader/cryptopp-cmake/issues/56
# https://stackoverflow.com/a/40152725
if(CMAKE_GENERATOR STREQUAL Xcode)
    set(CRYPTOPP_USE_INTERMEDIATE_OBJECTS_TARGET OFF)
endif()

# ============================================================================
# Compiler options
# ============================================================================

# Enable PIC for all target machines except 32-bit i386 due to register
# pressures. See https://github.com/abdes/cryptopp-cmake/issues/4
#
# Although the common practice with many Linux distros is to build static
# librarues with no PIC, we have a special case here for cryptopp:
#
# * We prefer to enable hardening of targets
#   (https://en.wikipedia.org/wiki/Address_space_layout_randomization), which at
#   a minimum requires the library and its using targets to be compiled and
#   linked with PIE
#
# * The wrapper DLL (https://www.cryptopp.com/wiki/Wrapper_DLL) usage scenario
#   requires that the object files in the static library be built with PIC,
#   otherwise, such object files cannot be linked into a DLL/shared library
#
# To satisfy both scenarios, we need to compile all source code with PIC enabled
# and link executables with PIE.
#
# TODO: make a specific test case for using wrapper DLL TODO: make a test case
# for -fPIE executable
if(NOT CRYPTOPP_I386)
    set(CMAKE_POSITION_INDEPENDENT_CODE 1)
endif()

set(CRYPTOPP_COMPILE_DEFINITIONS)
set(CRYPTOPP_COMPILE_OPTIONS)

if(CMAKE_CXX_COMPILER_ID STREQUAL "Intel")
    list(
        APPEND
        CRYPTOPP_COMPILE_OPTIONS
        -wd68
        -wd186
        -wd279
        -wd327
        -wd161
        -wd3180
    )
endif()

# Reflect any requests to disable features into the CXX compile definitions
foreach(
    feature
    ASM
    SSSE3
    SSE4
    CLMUL
    AESNI
    RDRAND
    RDSEED
    AVX
    AVX2
    SHA
    ARM_NEON
    ARM_ASIMD
    ARM_AES
    ARM_PMULL
    ARM_SHA
    ALTIVEC
    POWER7
    POWER8
    POWER9
)
    if(${CRYPTOPP_DISABLE_${feature}})
        message(
            STATUS
            "[cryptopp] -!!- CRYPTOPP_DISABLE_${feature}=${CRYPTOPP_DISABLE_${feature}}"
        )
        list(
            APPEND
            CRYPTOPP_COMPILE_DEFINITIONS
            "CRYPTOPP_DISABLE_${feature}=1"
        )
    endif()
endforeach()

# ##############################################################################

# Try to find a Posix compatible grep and sed. Solaris, Digital Unix, Tru64,
# HP-UX and a few others need tweaking

if(EXISTS /usr/xpg4/bin/grep)
    set(GREP_CMD /usr/xpg4/bin/grep)
elseif(EXISTS /usr/gnu/bin/grep)
    set(GREP_CMD /usr/gnu/bin/grep)
elseif(EXISTS /usr/linux/bin/grep)
    set(GREP_CMD /usr/linux/bin/grep)
else()
    set(GREP_CMD grep)
endif()

if(EXISTS /usr/xpg4/bin/sed)
    set(SED_CMD /usr/xpg4/bin/sed)
elseif(EXISTS /usr/gnu/bin/sed)
    set(SED_CMD /usr/gnu/bin/sed)
elseif(EXISTS /usr/linux/bin/sed)
    set(SED_CMD /usr/linux/bin/sed)
else()
    set(SED_CMD sed)
endif()

# ##############################################################################

# This check does not use any of the special test programs provided in crypto++,
# therefore, it should simply rely on what CMake provides.
function(check_compile_option opt var)
    if(DEFINED "${var}")
        return()
    endif()
    check_cxx_compiler_flag(${opt} ${var})
endfunction(check_compile_option)

function(check_compile_link_option opt var prog)
    if(DEFINED "${var}")
        return()
    endif()

    message(STATUS "[cryptopp] Performing Test ${var}")
    set(definitions ${CRYPTOPP_COMPILE_DEFINITIONS})
    list(TRANSFORM definitions PREPEND "-D")
    list(APPEND definitions ${opt})
    try_compile(
        COMMAND_SUCCESS
        ${CMAKE_BINARY_DIR}
        ${prog}
        COMPILE_DEFINITIONS ${definitions}
    )
    if(COMMAND_SUCCESS)
        set(${var} 1 CACHE INTERNAL "Test ${var}")
        message(STATUS "[cryptopp] Performing Test ${var} - Success")
    else()
        set(${var} 0 CACHE INTERNAL "Test ${var}")
        message(STATUS "[cryptopp] Performing Test ${var} - Failed")
    endif()
endfunction(check_compile_link_option)

# ##############################################################################

function(check_target_architecture output pattern)
    # The -dumpmachine does not work with MSVC, so we do detection using some C
    # code that will leverage the compiler definitions for the target
    # architecture.
    include(TargetArch)
    target_architecture(target_arch)
    string(TOLOWER ${target_arch} target_arch)
    if("${target_arch}" MATCHES ${pattern})
        message(
            STATUS
            "[cryptopp] Target architecture detected as: ${target_arch} -> ${output}"
        )
        set(${output} TRUE PARENT_SCOPE)
    else()
        set(${output} FALSE PARENT_SCOPE)
    endif()
endfunction()

if(APPLE)
    message(
        STATUS
        "[cryptopp]     CMAKE_OSX_ARCHITECTURES : ${CMAKE_OSX_ARCHITECTURES}"
    )
endif()
message(
    STATUS
    "[cryptopp] CMAKE_HOST_SYSTEM_PROCESSOR : ${CMAKE_HOST_SYSTEM_PROCESSOR}"
)
message(
    STATUS
    "[cryptopp]      CMAKE_SYSTEM_PROCESSOR : ${CMAKE_SYSTEM_PROCESSOR}"
)

check_target_architecture(CRYPTOPP_AMD64 "(x86_64|amd64)")
check_target_architecture(CRYPTOPP_CYGWIN "cygwin")
check_target_architecture(CRYPTOPP_I386 "^i.86$")
check_target_architecture(CRYPTOPP_MINGW32 "^mingw32")
check_target_architecture(CRYPTOPP_MINGW64 "(w64-mingw32|mingw64)")
check_target_architecture(CRYPTOPP_ARMV8 "(armv8|arm64|aarch32|aarch64)")
check_target_architecture(CRYPTOPP_ARM32 "(^arm$|arm32|armhf|arm7l|eabihf)")
check_target_architecture(CRYPTOPP_PPC32 "^(powerpc|ppc)")
check_target_architecture(CRYPTOPP_PPC64 "^ppc64")

# Cleanup 32/64 bit
if(CRYPTOPP_AMD64)
    set(CRYPTOPP_I386 0)
endif()

if(CRYPTOPP_ARMV8)
    set(CRYPTOPP_ARM32 0)
endif()

if(CRYPTOPP_PPC64)
    set(CRYPTOPP_PPC32 0)
endif()
# ##############################################################################

# Test SunCC for a string like 'CC: Sun C++ 5.13 SunOS_i386'
if(NOT CRYPTOPP_SOLARIS AND NOT MSVC)
    execute_process(
        COMMAND sh -c "${CMAKE_CXX_COMPILER} -V 2>&1"
        COMMAND ${GREP_CMD} -i -c "SunOS"
        OUTPUT_VARIABLE CRYPTOPP_SOLARIS
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
endif()

# Test GCC for a string like 'i386-pc-solaris2.11'
if(NOT CRYPTOPP_SOLARIS AND NOT MSVC)
    execute_process(
        COMMAND sh -c "${CMAKE_CXX_COMPILER} -dumpmachine 2>&1"
        COMMAND ${GREP_CMD} -i -c "Solaris"
        OUTPUT_VARIABLE CRYPTOPP_SOLARIS
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
endif()

# Fixup PowerPC. If both 32-bit and 64-bit use 64-bit.
if(CRYPTOPP_PPC32 AND CRYPTOPP_PPC64)
    unset(CRYPTOPP_PPC32)
endif()

# Fixup for xlC compiler. -dumpmachine fails so we miss PowerPC TODO: something
# better than proxying the platform via compiler Must use CMAKE_CXX_COMPILER
# here due to XLC 13.1 and LLVM front-end.
if(CMAKE_CXX_COMPILER MATCHES "xlC")
    message(STATUS "[cryptopp] -- Fixing platform due to IBM xlC")
    set(CRYPTOPP_PPC64 1)
endif()

# DumpMachine SunCC style
if(CMAKE_CXX_COMPILER_ID STREQUAL "SunPro")
    # SunCC is 32-bit, but it builds both 32 and 64 bit. Use
    execute_process(
        COMMAND sh -c "${CMAKE_CXX_COMPILER} -V 2>&1"
        COMMAND ${GREP_CMD} -i -c "Sparc"
        OUTPUT_VARIABLE CRYPTOPP_SPARC
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    execute_process(
        COMMAND sh -c "${CMAKE_CXX_COMPILER} -V 2>&1"
        COMMAND ${GREP_CMD} -i -c -E "i386|i86"
        OUTPUT_VARIABLE CRYPTOPP_I386
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    execute_process(
        COMMAND isainfo -k
        COMMAND ${GREP_CMD} -i -c "i386"
        OUTPUT_VARIABLE KERNEL_I386
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    execute_process(
        COMMAND isainfo -k
        COMMAND ${GREP_CMD} -i -c "amd64"
        OUTPUT_VARIABLE KERNEL_AMD64
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    execute_process(
        COMMAND isainfo -k
        COMMAND ${GREP_CMD} -i -c "Sparc"
        OUTPUT_VARIABLE KERNEL_SPARC
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    execute_process(
        COMMAND isainfo -k
        COMMAND ${GREP_CMD} -i -c -E "UltraSarc|Sparc64|SparcV9"
        OUTPUT_VARIABLE KERNEL_SPARC64
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
endif()

# ##############################################################################

if(MSVC)
    set(CRYPTOPP_MSVC_COMPILE_OPTIONS)

    # TODO: what about ICC and LLVM on Windows?

    if(CMAKE_SYSTEM_VERSION MATCHES "10\\.0.*")
        # https://docs.microsoft.com/en-us/cpp/porting/modifying-winver-and-win32-winnt
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "_WIN32_WINNT=0x0A00")
    endif()
    # winapifamily.h is missing on AppVeyor machines
    include(CheckIncludeFileCXX)
    check_include_file_cxx("winapifamily.h" HAVE_WINAPIFAMILY_H)
    if(HAVE_WINAPIFAMILY_H AND (NOT (USE_CCACHE)))
        list(APPEND CRYPTOPP_MSVC_COMPILE_OPTIONS "/FIwinapifamily.h")
    endif()

    list(APPEND CRYPTOPP_MSVC_COMPILE_OPTIONS "/GR")
    list(APPEND CRYPTOPP_MSVC_COMPILE_OPTIONS "/MP")
    list(APPEND CRYPTOPP_MSVC_COMPILE_OPTIONS "/EHsc")
endif()

# IBM XLC compiler options for AIX and Linux. Must use CMAKE_CXX_COMPILER here
# due to XLC 13.1 and LLVM front-end.
if(CMAKE_CXX_COMPILER MATCHES "xlC")
    # CheckCompileLinkOption("-qxlcompatmacros" CRYPTOPP_XLC_COMPAT
    # "${TEST_CXX_FILE}") if (CRYPTOPP_XLC_COMPAT) list(APPEND
    # CRYPTOPP_COMPILE_OPTIONS "-qxlcompatmacros") endif ()

    check_compile_link_option("-qrtti" CRYPTOPP_PPC_RTTI "${TEST_CXX_FILE}")
    if(CRYPTOPP_PPC_RTTI)
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-qrtti")
    endif()

    check_compile_link_option("-qmaxmem=-1"   CRYPTOPP_PPC_MAXMEM
                              "${TEST_CXX_FILE}"
    )
    if(CRYPTOPP_PPC_MAXMEM)
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-qmaxmem=-1")
    endif()

    check_compile_link_option("-qthreaded"   CRYPTOPP_PPC_THREADED
                              "${TEST_CXX_FILE}"
    )
    if(CRYPTOPP_PPC_THREADED)
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-qthreaded")
    endif()
endif()

# Solaris specific
if(CRYPTOPP_SOLARIS)
    # SunCC needs -template=no%extdef
    if(CMAKE_CXX_COMPILER_ID STREQUAL "SunPro")
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-template=no%extdef")
    endif()

    # SunCC needs -xregs=no%appl on Sparc (not x86) for libraries (not test
    # program) TODO: wire this up properly
    if(
        CMAKE_CXX_COMPILER_ID STREQUAL "SunPro"
        AND (CRYPTOPP_SPARC OR CRYPTOPP_SPARC64)
    )
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-xregs=no%appl")
    endif()

    # GCC needs to enable use of '/' for division in the assembler
    if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
        list(APPEND CRYPTOPP_COMPILE_OPTIONS "-Wa,--divide")
    endif()
endif()

# ------------------------------------------------------------------------------
# https://github.com/abdes/cryptopp-cmake/issues/3
#
# If on Linux with GCC 12 or above, we need to add "-fno-devirtualize" flag to
# workaround a specific issue introduced by gcc-12.
if(
    CMAKE_SYSTEM_NAME STREQUAL "Linux"
    AND CMAKE_CXX_COMPILER_ID STREQUAL "GNU"
    AND CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL "12"
)
    list(APPEND CRYPTOPP_COMPILE_OPTIONS "-fno-devirtualize")
endif()
# ------------------------------------------------------------------------------

# ============================================================================
# Sources & headers
# ============================================================================

# Library sources. You can use the GNUmakefile to generate the list: `make
# sources`. Makefile sorted them at
# http://github.com/weidai11/cryptopp/pull/426.
list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR})
include(sources)

# ==============================================================================
# Compiler and architecture specific options
# ==============================================================================

# ##############################################################################
# X86/X32/X64 Options               #####
# ##############################################################################

if(
    (
        CRYPTOPP_I386
        OR CRYPTOPP_AMD64
        OR CRYPTOPP_MINGW32
        OR CRYPTOPP_MINGW64
        OR CRYPTOPP_CYGWIN
    )
    AND NOT MSVC
)
    if(CMAKE_CXX_COMPILER_ID STREQUAL "SunPro")
        set(sse2_flag "-xarch=sse2")
        set(sse3_flag "-xarch=sse3")
        set(ssse3_flag "-xarch=ssse3")
        set(sse41_flag "-xarch=sse4_1")
        set(sse42_flag "-xarch=sse4_2")
        set(clmul_flag "-xarch=aes")
        set(aesni_flag "-xarch=aes")
        set(avx_flag "-xarch=avx")
        set(avx2_flag "-xarch=avx2")
        set(shani_flag "-xarch=sha")
    else()
        set(sse2_flag "-msse2")
        set(sse3_flag "-msse3")
        set(ssse3_flag "-mssse3")
        set(sse41_flag "-msse4.1")
        set(sse42_flag "-msse4.2")
        set(clmul_flag "-mpclmul")
        set(aesni_flag "-maes")
        set(avx_flag "-mavx")
        set(avx2_flag "-mavx2")
        set(shani_flag "-msha")
    endif()

    # For Darwin and a GCC port compiler, we need to check for -Wa,-q first.
    # -Wa,-q is a GCC option, and it tells GCC to use the Clang Integrated
    # Assembler. We need LLVM's assembler because GAS is too old on Apple
    # platforms. GAS will not assemble modern ISA, like AVX or AVX2.
    if(APPLE)
        check_compile_link_option("-Wa,-q"     CRYPTOPP_X86_WAQ
                                  "${TEST_PROG_DIR}/test_x86_sse2.cpp"
        )

        if(CRYPTOPP_X86_WAQ)
            list(APPEND CRYPTOPP_COMPILE_OPTIONS "-Wa,-q")
        endif()
    endif()

    check_compile_link_option(${sse2_flag} CRYPTOPP_HAVE_SSE2
                              "${TEST_PROG_DIR}/test_x86_sse2.cpp"
    )
    if(CRYPTOPP_HAVE_SSE2)
        list(APPEND CRYPTOPP_CHACHA_FLAGS "${sse2_flag}")
        list(APPEND CRYPTOPP_SUN_LDFLAGS "${sse2_flag}")

        # Need SSE2 or higher for these tests

        check_compile_link_option(${sse3_flag} CRYPTOPP_HAVE_SSE3
                                  "${TEST_PROG_DIR}/test_x86_sse3.cpp"
        )
        if(NOT CRYPTOPP_HAVE_SSE3)
            set(sse3_flag "")
        endif()

        check_compile_link_option(${ssse3_flag} CRYPTOPP_HAVE_SSSE3
                                  "${TEST_PROG_DIR}/test_x86_ssse3.cpp"
        )
        if(CRYPTOPP_HAVE_SSSE3)
            list(APPEND CRYPTOPP_ARIA_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_CHAM_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_GCM_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_KECCAK_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_LEA_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_LSH256_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_LSH512_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_SIMON128_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_SM4_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_SPECK128_FLAGS "${ssse3_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${ssse3_flag}")
        else()
            set(ssse3_flag "")
        endif()

        check_compile_link_option(${sse41_flag} CRYPTOPP_HAVE_SSE41
                                  "${TEST_PROG_DIR}/test_x86_sse41.cpp"
        )
        if(CRYPTOPP_HAVE_SSE41)
            list(APPEND CRYPTOPP_AES_FLAGS "${sse41_flag}")
            list(APPEND CRYPTOPP_BLAKE2B_FLAGS "${sse41_flag}")
            list(APPEND CRYPTOPP_BLAKE2S_FLAGS "${sse41_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${sse41_flag}")
        else()
            set(sse41_flag "")
        endif()

        check_compile_link_option(${sse42_flag} CRYPTOPP_HAVE_SSE42
                                  "${TEST_PROG_DIR}/test_x86_sse42.cpp"
        )
        if(CRYPTOPP_HAVE_SSE42)
            list(APPEND CRYPTOPP_CRC_FLAGS "${sse42_flag}")
            list(APPEND CRYPTOPP_SHA_FLAGS "${sse42_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${sse42_flag}")
        else()
            set(sse42_flag "")
        endif()

        check_compile_link_option(${clmul_flag} CRYPTOPP_HAVE_CLMUL
                                  "${TEST_PROG_DIR}/test_x86_clmul.cpp"
        )
        if(CRYPTOPP_HAVE_CLMUL)
            list(APPEND CRYPTOPP_GCM_FLAGS "${clmul_flag}")
            list(APPEND CRYPTOPP_GF2N_FLAGS "${clmul_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${clmul_flag}")
        else()
            set(clmul_flag "")
        endif()

        check_compile_link_option(${aesni_flag} CRYPTOPP_HAVE_AESNI
                                  "${TEST_PROG_DIR}/test_x86_aes.cpp"
        )
        if(CRYPTOPP_HAVE_CLMUL)
            list(APPEND CRYPTOPP_AES_FLAGS "${aesni_flag}")
            list(APPEND CRYPTOPP_SM4_FLAGS "${aesni_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${aesni_flag}")
        else()
            set(aesni_flag "")
        endif()

        check_compile_link_option(${avx_flag} CRYPTOPP_HAVE_AVX
                                  "${TEST_PROG_DIR}/test_x86_avx.cpp"
        )
        if(CRYPTOPP_HAVE_CLMUL)
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${avx_flag}")
        else()
            set(avx_flag "")
        endif()

        check_compile_link_option(${avx2_flag} CRYPTOPP_HAVE_AVX2
                                  "${TEST_PROG_DIR}/test_x86_avx2.cpp"
        )
        if(CRYPTOPP_HAVE_CLMUL)
            list(APPEND CRYPTOPP_CHACHA_AVX2_FLAGS "${avx2_flag}")
            list(APPEND CRYPTOPP_LSH256_AVX2_FLAGS "${avx2_flag}")
            list(APPEND CRYPTOPP_LSH512_AVX2_FLAGS "${avx2_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${avx2_flag}")
        else()
            set(avx2_flag "")
        endif()

        check_compile_link_option(${shani_flag} CRYPTOPP_HAVE_SHANI
                                  "${TEST_PROG_DIR}/test_x86_sha.cpp"
        )
        if(CRYPTOPP_HAVE_SHANI)
            list(APPEND CRYPTOPP_SHA_FLAGS "${shani_flag}")
            list(APPEND CRYPTOPP_SUN_LDFLAGS "${shani_flag}")
        else()
            set(shani_flag "")
        endif()

        if(NOT sse3_flag)
            list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_SSE3=1")
        elseif(NOT ssse3_flag)
            list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_SSSE3=1")
        elseif(NOT sse41_flag)
            list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_SSE41=1")
        elseif(NOT sse42_flag)
            list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_SSE42=1")
        endif()

        if(sse42_flag)
            # Unusual GCC/Clang on Macports. It assembles AES, but not CLMUL.
            # test_x86_clmul.s:15: no such instruction: 'pclmulqdq $0, %xmm1,%xmm0'
            if(NOT clmul_flag)
                list(
                    APPEND
                    CRYPTOPP_COMPILE_DEFINITIONS
                    "CRYPTOPP_DISABLE_CLMUL=1"
                )
            endif()
            if(NOT aesni_flag)
                list(
                    APPEND
                    CRYPTOPP_COMPILE_DEFINITIONS
                    "CRYPTOPP_DISABLE_AESNI=1"
                )
            endif()

            if(NOT avx_flag)
                list(
                    APPEND
                    CRYPTOPP_COMPILE_DEFINITIONS
                    "CRYPTOPP_DISABLE_AVX=1"
                )
            elseif(NOT avx2_flag)
                list(
                    APPEND
                    CRYPTOPP_COMPILE_DEFINITIONS
                    "CRYPTOPP_DISABLE_AVX2=1"
                )
            endif()
            # SHANI independent of AVX per GH #1045
            if(NOT shani_flag)
                list(
                    APPEND
                    CRYPTOPP_COMPILE_DEFINITIONS
                    "CRYPTOPP_DISABLE_SHANI=1"
                )
            endif()
        endif()

        # Drop to SSE2 if available
        if(NOT gcm_flag)
            set(gcm_flag ${sse2_flag})
        endif()
    else()
        set(sse2_flag "")
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ASM=1")
    endif()

    if(CMAKE_CXX_COMPILER_ID STREQUAL "Intel")
        list(
            APPEND
            CRYPTOPP_COMPILE_OPTIONS
            -wd68
            -wd186
            -wd279
            -wd327
            -wd161
            -wd3180
        )
        if(CMAKE_CXX_COMPILER_VERSION VERSION_LESS "11.1")
            # "internal error: backend signals" occurs on some x86 inline assembly
            # with ICC 9 and some x64 inline assembly with ICC 11.0. If you want to
            # use Crypto++'s assembly code with ICC, try enabling it on individual
            # files
            list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ASM=1")
        endif()
    endif()

    # Allow use of "/" operator for GNU Assembler.
    # http://sourceware.org/bugzilla/show_bug.cgi?id=4572
    list(
        FIND
        CRYPTOPP_COMPILE_DEFINITIONS
        "CRYPTOPP_DISABLE_ASM=1"
        has_disable_asm
    )
    if(has_disable_asm EQUAL -1)
        if(CMAKE_SYSTEM MATCHES "SunOS" AND CMAKE_CXX_COMPILER_ID MATCHES "GNU")
            list(APPEND CRYPTOPP_COMPILE_OPTIONS -Wa,--divide)
        endif()
    endif()
endif()

# ##############################################################################
# ARM A-32 and NEON                #####
# ##############################################################################

if(CRYPTOPP_ARM32 AND NOT CRYPTOPP_DISABLE_ARM_NEON)
    # Clang needs an option to include <arm_neon.h>
    check_compile_link_option(
      "-DCRYPTOPP_ARM_NEON_HEADER=1 -march=armv7-a -mfpu=neon"
      CRYPTOPP_ARM_NEON_HEADER   "${TEST_PROG_DIR}/test_arm_neon_header.cpp"
    )
    if(CRYPTOPP_ARM_NEON_HEADER)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_ARM_NEON_HEADER=1")
    endif()

    list(APPEND arm_neon_flags -march=armv7-a -mfpu=neon)
    check_compile_link_option("-march=armv7-a -mfpu=neon"   CRYPTOPP_HAVE_ARM_NEON
                              "${TEST_PROG_DIR}/test_arm_neon.cpp"
    )
    if(CRYPTOPP_HAVE_ARM_NEON)
        list(APPEND CRYPTOPP_NEON_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_ARIA_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_GCM_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_BLAKE2B_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_BLAKE2S_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_CHACHA_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_CHAM_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_LEA_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_SIMON128_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_SPECK128_FLAGS -march=armv7-a -mfpu=neon)
        list(APPEND CRYPTOPP_SM4_FLAGS -march=armv7-a -mfpu=neon)
    else()
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ARM_NEON=1")
    endif()
endif()

# Cryptogams source files. We couple to ARMv7 and NEON due to SHA using NEON.
# Limit to Linux. The source files target the GNU assembler. Also see
# https://www.cryptopp.com/wiki/Cryptogams.
if(
    CRYPTOPP_ARM32
    AND (CMAKE_SYSTEM_NAME STREQUAL "Linux" OR ANDROID)
    AND NOT CRYPTOPP_DISABLE_ARM_NEON
    AND NOT CRYPTOPP_DISABLE_ASM
)
    list(
        APPEND
        cryptopp_SOURCES_ASM
        ${cryptopp_SOURCE_DIR}/aes_armv4.S
        ${cryptopp_SOURCE_DIR}/sha1_armv4.S
        ${cryptopp_SOURCE_DIR}/sha256_armv4.S
        ${cryptopp_SOURCE_DIR}/sha512_armv4.S
    )
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        list(APPEND CRYPTOGAMS_ARM_FLAG -march=armv7-a)
        list(APPEND CRYPTOGAMS_ARM_THUMB_FLAG -march=armv7-a -mthumb)
    else()
        # -mfpu=auto due to https://github.com/weidai11/cryptopp/issues/1094
        list(APPEND CRYPTOGAMS_ARM_FLAG -march=armv7-a)
        list(APPEND CRYPTOGAMS_ARM_THUMB_FLAG -march=armv7-a)
    endif()
endif()

# ##############################################################################
# Aach32 and Aarch64               #####
# ##############################################################################

if(CRYPTOPP_ARMV8)
    check_compile_link_option(
      "-DCRYPTOPP_ARM_NEON_HEADER=1"   CRYPTOPP_ARM_NEON_HEADER
      "${TEST_PROG_DIR}/test_arm_neon_header.cpp"
    )
    if(CRYPTOPP_ARM_NEON_HEADER)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_ARM_NEON_HEADER=1")
    endif()

    check_compile_link_option(
      "-DCRYPTOPP_ARM_ACLE_HEADER=1 -march=armv8-a"   CRYPTOPP_ARM_ACLE_HEADER
      "${TEST_PROG_DIR}/test_arm_acle_header.cpp"
    )
    if(CRYPTOPP_ARM_ACLE_HEADER)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_ARM_ACLE_HEADER=1")
    endif()

    check_compile_link_option("-march=armv8-a"   CRYPTOPP_ARM_SIMD
                              "${TEST_PROG_DIR}/test_arm_asimd.cpp"
    )
    if(CRYPTOPP_ARM_SIMD)
        list(APPEND CRYPTOPP_ASIMD_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_ARIA_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_BLAKE2B_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_BLAKE2S_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_CHACHA_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_CHAM_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_LEA_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_NEON_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_SIMON128_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_SPECK128_FLAGS -march=armv8-a)
        list(APPEND CRYPTOPP_SM4_FLAGS -march=armv8-a)

        check_compile_link_option("-march=armv8-a+crc"     CRYPTOPP_HAVE_ARM_CRC32
                                  "${TEST_PROG_DIR}/test_arm_crc.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_CRC32)
            list(APPEND CRYPTOPP_CRC_FLAGS -march=armv8-a+crc)
        else()
            list(
                APPEND
                CRYPTOPP_COMPILE_DEFINITIONS
                "CRYPTOPP_DISABLE_ARM_CRC32=1"
            )
        endif()

        check_compile_link_option("-march=armv8-a+crypto"     CRYPTOPP_HAVE_ARM_AES
                                  "${TEST_PROG_DIR}/test_arm_aes.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_AES)
            list(APPEND CRYPTOPP_AES_FLAGS -march=armv8-a+crypto)
        else()
            list(
                APPEND
                CRYPTOPP_COMPILE_DEFINITIONS
                "CRYPTOPP_DISABLE_ARM_AES=1"
            )
        endif()

        check_compile_link_option("-march=armv8-a+crypto"     CRYPTOPP_HAVE_ARM_PMULL
                                  "${TEST_PROG_DIR}/test_arm_pmull.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_PMULL)
            list(APPEND CRYPTOPP_GCM_FLAGS -march=armv8-a+crypto)
            list(APPEND CRYPTOPP_GF2N_FLAGS -march=armv8-a+crypto)
        else()
            list(
                APPEND
                CRYPTOPP_COMPILE_DEFINITIONS
                "CRYPTOPP_DISABLE_ARM_PMULL=1"
            )
        endif()

        check_compile_link_option("-march=armv8-a+crypto"     CRYPTOPP_HAVE_ARM_SHA1
                                  "${TEST_PROG_DIR}/test_arm_sha1.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_SHA1)
            list(APPEND CRYPTOPP_SHA_FLAGS -march=armv8-a+crypto)
        else()
            list(
                APPEND
                CRYPTOPP_COMPILE_DEFINITIONS
                "CRYPTOPP_DISABLE_ARM_SHA1=1"
            )
        endif()

        check_compile_link_option("-march=armv8-a+crypto"     CRYPTOPP_HAVE_ARM_SHA2
                                  "${TEST_PROG_DIR}/test_arm_sha256.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_SHA2)
            list(APPEND CRYPTOPP_SHA_FLAGS -march=armv8-a+crypto)
        else()
            list(
                APPEND
                CRYPTOPP_COMPILE_DEFINITIONS
                "CRYPTOPP_DISABLE_ARM_SHA2=1"
            )
        endif()

        check_compile_link_option("-march=armv8.4-a+sm3"     CRYPTOPP_HAVE_ARM_SM3
                                  "${TEST_PROG_DIR}/test_arm_sm3.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_SM3)
            list(APPEND CRYPTOPP_SM3_FLAGS -march=armv8.4-a+sm3)
            list(APPEND CRYPTOPP_SM4_FLAGS -march=armv8.4-a+sm3)
        else()
            # list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ARM_SM3=1")
            # list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ARM_SM4=1")
        endif()

        check_compile_link_option("-march=armv8.4-a+sha3"     CRYPTOPP_HAVE_ARM_SHA3
                                  "${TEST_PROG_DIR}/test_arm_sha3.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_SHA3)
            list(APPEND CRYPTOPP_SHA3_FLAGS -march=armv8.4-a+sha3)
        else()
            # list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ARM_SHA3=1")
        endif()

        check_compile_link_option(
          "-march=armv8.4-a+sha512"     CRYPTOPP_HAVE_ARM_SHA512
          "${TEST_PROG_DIR}/test_arm_sha512.cpp"
        )
        if(CRYPTOPP_HAVE_ARM_SHA512)
            list(APPEND CRYPTOPP_SHA3_FLAGS -march=armv8.4-a+sha512)
        else()
            # list(APPEND CRYPTOPP_COMPILE_DEFINITIONS
            # "CRYPTOPP_DISABLE_ARM_SHA512=1")
        endif()
    else()
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ASM=1")
    endif()
endif()

# ##############################################################################
# PowerPC                     #####
# ##############################################################################

# PowerPC and PowerPC64. Altivec is available with POWER4 with GCC and POWER6
# with XLC. The tests below are crafted for IBM XLC and the LLVM front-end.
# XLC/LLVM only supplies POWER8 so we have to set the flags for XLC/LLVM to
# POWER8. I've got a feeling LLVM is going to cause trouble.

if(CRYPTOPP_PPC32 OR CRYPTOPP_PPC64)
    # IBM XL C/C++ has the -qaltivec flag really screwed up. We can't seem to get
    # it enabled without an -qarch= option. And -qarch= produces an error on later
    # versions of the compiler. The only thing that seems to work consistently is
    # -qarch=auto. -qarch=auto is equivalent to GCC's -march=native, which we
    # don't really want.

    # XLC requires -qaltivec in addition to Arch or CPU option
    if(CMAKE_CXX_COMPILER MATCHES "xlC")
        # list(APPEND CRYPTOPP_POWER9_FLAGS -qarch=pwr9 -qaltivec)
        list(APPEND CRYPTOPP_POWER8_FLAGS -qarch=pwr8 -qaltivec)
        list(APPEND CRYPTOPP_POWER7_VSX_FLAGS -qarch=pwr7 -qvsx -qaltivec)
        list(APPEND CRYPTOPP_POWER7_PWR_FLAGS -qarch=pwr7 -qaltivec)
        list(APPEND CRYPTOPP_ALTIVEC_FLAGS -qarch=auto -qaltivec)
    else()
        # list(APPEND CRYPTOPP_POWER9_FLAGS -mcpu=power9)
        list(APPEND CRYPTOPP_POWER8_FLAGS -mcpu=power8)
        list(APPEND CRYPTOPP_POWER7_VSX_FLAGS -mcpu=power7 -mvsx)
        list(APPEND CRYPTOPP_POWER7_PWR_FLAGS -mcpu=power7)
        list(APPEND CRYPTOPP_ALTIVEC_FLAGS -maltivec)
    endif()

    # GCC 10 is giving us trouble in CPU_ProbePower9() and CPU_ProbeDARN(). GCC is
    # generating POWER9 instructions on POWER8 for ppc_power9.cpp. The compiler
    # folks did not think through the consequences of requiring us to use
    # -mcpu=power9 to unlock the ISA. Epic fail.
    # https:#github.com/weidai11/cryptopp/issues/986
    set(POWER9_FLAG)

    # ############################################################################
    # Looking for a POWER8 option

    string(REPLACE ";" " " CRYPTOPP_POWER8_FLAGS_STR "${CRYPTOPP_POWER8_FLAGS}")
    check_compile_link_option(${CRYPTOPP_POWER8_FLAGS_STR} CRYPTOPP_HAVE_POWER8
                              "${TEST_PROG_DIR}/test_ppc_power8.cpp"
    )
    if(CRYPTOPP_HAVE_POWER8)
        list(APPEND CRYPTOPP_AES_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_BLAKE2B_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_CRC_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_GCM_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_GF2N_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_LEA_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        list(APPEND CRYPTOPP_SHA_FLAGS ${CRYPTOPP_POWER8_FLAGS})
        # list(APPEND CRYPTOPP_SHACAL2_FLAGS ${CRYPTOPP_POWER8_FLAGS})
    else()
        set(CRYPTOPP_POWER8_FLAGS)
    endif()

    # ############################################################################
    # Looking for a POWER7 option

    # GCC needs -mvsx for Power7 to enable 64-bit vector elements. XLC provides
    # 64-bit vector elements without an option.

    string(
        REPLACE
        ";"
        " "
        CRYPTOPP_POWER7_VSX_FLAGS_STR
        "${CRYPTOPP_POWER7_VSX_FLAGS}"
    )
    check_compile_link_option(
      ${CRYPTOPP_POWER7_VSX_FLAGS_STR} CRYPTOPP_HAVE_POWER7_VSX
      "${TEST_PROG_DIR}/test_ppc_power7.cpp"
    )
    if(CRYPTOPP_HAVE_POWER7_VSX)
        list(APPEND CRYPTOPP_POWER7_FLAGS ${CRYPTOPP_POWER7_VSX_FLAGS})
    else()
        string(
            REPLACE
            ";"
            " "
            CRYPTOPP_POWER7_PWR_FLAGS_STR
            "${CRYPTOPP_POWER7_PWR_FLAGS}"
        )
        check_compile_link_option(
          ${CRYPTOPP_POWER7_PWR_FLAGS_STR} CRYPTOPP_HAVE_POWER7_PWR
          "${TEST_PROG_DIR}/test_ppc_power7.cpp"
        )
        if(CRYPTOPP_HAVE_POWER7_PWR)
            list(APPEND CRYPTOPP_POWER7_FLAGS ${CRYPTOPP_POWER7_PWR_FLAGS})
        else()
            set(CRYPTOPP_POWER7_FLAGS)
        endif()
    endif()

    # ############################################################################
    # Looking for an Altivec option

    string(
        REPLACE
        ";"
        " "
        CRYPTOPP_ALTIVEC_FLAGS_STR
        "${CRYPTOPP_ALTIVEC_FLAGS}"
    )
    check_compile_link_option(${CRYPTOPP_ALTIVEC_FLAGS_STR} CRYPTOPP_HAVE_ALTIVEC
                              "${TEST_PROG_DIR}/test_ppc_altivec.cpp"
    )
    if(CRYPTOPP_HAVE_ALTIVEC)
        list(APPEND CRYPTOPP_BLAKE2S_FLAGS ${CRYPTOPP_ALTIVEC_FLAGS})
        list(APPEND CRYPTOPP_CHACHA_FLAGS ${CRYPTOPP_ALTIVEC_FLAGS})
        list(APPEND CRYPTOPP_SPECK128_FLAGS ${CRYPTOPP_ALTIVEC_FLAGS})
        list(APPEND CRYPTOPP_SIMON128_FLAGS ${CRYPTOPP_ALTIVEC_FLAGS})
    else()
        set(CRYPTOPP_ALTIVEC_FLAGS)
    endif()

    # ############################################################################
    # Fixups for algorithms that can drop to a lower ISA, if needed

    # Drop to Altivec if higher Power is not available
    if(CRYPTOPP_ALTIVEC_FLAGS AND NOT CRYPTOPP_GCM_FLAGS)
        list(APPEND CRYPTOPP_CGM_FLAGS ${CRYPTOPP_ALTIVEC_FLAGS})
    endif()

    # ############################################################################
    # Fixups for missing ISAs

    if(NOT CRYPTOPP_ALTIVEC_FLAGS)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ALTIVEC=1")
    elseif(NOT CRYPTOPP_POWER7_FLAGS)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_POWER7=1")
    elseif(NOT CRYPTOPP_POWER8_FLAGS)
        list(APPEND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_POWER8=1")
        # elseif(NOT CRYPTOPP_POWER9_FLAGS) list(APPEND CRYPTOPP_COMPILE_DEFINITIONS
        # "CRYPTOPP_DISABLE_POWER9=1")
    endif()

    # IBM XL C++ compiler
    if(CMAKE_CXX_COMPILER MATCHES "xlC")
        list(APPEND CRYPTOPP_COMPILE_OPTIONS -qmaxmem=-1)
        # http://www-01.ibm.com/support/docview.wss?uid=swg21007500
        list(APPEND CRYPTOPP_COMPILE_OPTIONS -qrtti)

        # Disable IBM XL C++ "1500-036: (I) The NOSTRICT option (default at OPT(3))
        # has the potential to alter the semantics of a program."
        check_compile_link_option("-qsuppress=1500-036"     CRYPTOPP_HAVE_QSUPPRESS
                                  "${TEST_PROG_DIR}/test_cxx.cpp"
        )
        if(CRYPTOPP_HAVE_QSUPPRESS)
            list(APPEND CRYPTOPP_COMPILE_OPTIONS -qsuppress=1500-036)
        endif()
    endif()
endif()

# Remove unneeded arch specific files to speed build time.
if(NOT CRYPTOPP_PPC32 AND NOT CRYPTOPP_PPC64)
    list(FILTER cryptopp_SOURCES EXCLUDE REGEX "_ppc.cpp")
endif()
if(NOT CRYPTOPP_ARM32 AND NOT CRYPTOPP_ARMV8)
    list(FILTER cryptopp_SOURCES EXCLUDE REGEX "arm_|neon_|_armv4.S")
endif()
if(NOTCRYPTOPP_I386 AND NOT CRYPTOPP_AMD64)
    list(FILTER cryptopp_SOURCES EXCLUDE REGEX "sse_|_sse.cpp|_avx.cpp")
endif()

# If ASM is disabled we can remove the SIMD files, too.
list(FIND CRYPTOPP_COMPILE_DEFINITIONS "CRYPTOPP_DISABLE_ASM=1" has_disable_asm)
if(NOT has_disable_asm EQUAL -1)
    list(
        FILTER cryptopp_SOURCES
        EXCLUDE
        REGEX
            "arm_|ppc_|neon_|sse_|_sse.cpp|_avx.cpp|_ppc.cpp|_simd.cpp|_armv4.S"
    )
    # But keep sse_simd.cpp or we get undefined symbols at link
    list(PREPEND cryptopp_SOURCES "${cryptopp_SOURCE_DIR}/sse_simd.cpp")
endif()

# Apply compiler options to SIMD source files
set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/aria_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_ARIA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/blake2s_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_BLAKE2S_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/blake2b_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_BLAKE2B_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/aria_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_ARIA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/chacha_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_CHACHA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/chacha_avx.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_CHACHA_AVX2_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/cham_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_CHAM_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/crc_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_CRC_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/darn.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_DARN_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/donna_sse.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_DONNA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/gcm_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_GCM_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/gf2n_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_GF2N_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/keccak_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_KECCAK_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/lea_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_LEA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/lsh256_sse.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_LSH256_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/lsh256_avx.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_LSH256_AVX2_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/lsh512_sse.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_LSH512_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/lsh512_avx.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_LSH512_AVX2_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/neon_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_NEON_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/ppc_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_PPC_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/rijndael_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_AES_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/sha_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SHA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/sha3_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SHA3_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/shacal2_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SHA_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/simon128_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SIMON128_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/speck128_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SPECK128_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/sm3_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SM3_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/sm4_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SM4_FLAGS}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/${file}
    PROPERTIES COMPILE_OPTIONS "${xlc_compile_options}"
)

set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/aes_armv4.S
    PROPERTIES COMPILE_OPTIONS "${CRYPTOGAMS_ARM_THUMB_FLAG}"
)
foreach(file sha1_armv4.S sha256_armv4.S sha512_armv4.S)
    set_source_files_properties(
        ${cryptopp_SOURCE_DIR}/${file}
        PROPERTIES COMPILE_OPTIONS "${CRYPTOGAMS_ARM_FLAG}"
    )
endforeach()

# IBM XLC -O3 optimization bug
if(CMAKE_CXX_COMPILER MATCHES "xlC")
    foreach(file sm3.cpp donna_32.cpp donna_64.cpp)
        get_source_file_property(
            xlc_compile_options
            ${cryptopp_SOURCE_DIR}/$file
            COMPILE_OPTIONS
        )
        list(REMOVE_ITEM xlc_compile_options "-O3")
        list(APPEND xlc_compile_options "-O2")
        set_source_files_properties(
            ${cryptopp_SOURCE_DIR}/${file}
            PROPERTIES COMPILE_OPTIONS "${xlc_compile_options}"
        )
    endforeach()
endif()

# SSE2 on i686
set_source_files_properties(
    ${cryptopp_SOURCE_DIR}/sse_simd.cpp
    PROPERTIES COMPILE_OPTIONS "${CRYPTOPP_SSE2_FLAGS}"
)

# ------------------------------------------------------------------------------
# Compiler: MSVC
# ------------------------------------------------------------------------------
if(MSVC AND NOT CRYPTOPP_DISABLE_ASM)
    if(${CMAKE_GENERATOR_PLATFORM} MATCHES "ARM")
        message(
            STATUS
            "[cryptopp] Disabling ASM because ARM is specified as target platform."
        )
    else()
        enable_language(ASM_MASM)
        if(NOT CRYPTOPP_DISABLE_RDRAND)
            list(APPEND cryptopp_SOURCES_ASM ${cryptopp_SOURCE_DIR}/rdrand.asm)
            if(${CMAKE_GENERATOR} MATCHES "Visual Studio.*")
                # workaround https://github.com/abdes/cryptopp-cmake/issues/13
                set_source_files_properties(
                    ${cryptopp_SOURCE_DIR}/rdrand.asm
                    PROPERTIES COMPILE_OPTIONS "/Fo\$(IntDir)rdrand.asm.obj"
                )
            endif()
        endif()
        if(NOT CRYPTOPP_DISABLE_RDSEED)
            list(APPEND cryptopp_SOURCES_ASM ${cryptopp_SOURCE_DIR}/rdseed.asm)
        endif()
        if(CMAKE_SIZEOF_VOID_P EQUAL 8)
            list(
                APPEND
                cryptopp_SOURCES_ASM
                ${cryptopp_SOURCE_DIR}/x64dll.asm
                ${cryptopp_SOURCE_DIR}/x64masm.asm
            )
            set_source_files_properties(
                ${cryptopp_SOURCES_ASM}
                PROPERTIES COMPILE_DEFINITIONS "_M_X64"
            )
        else()
            set_source_files_properties(
                ${cryptopp_SOURCES_ASM}
                PROPERTIES COMPILE_DEFINITIONS "_M_X86" COMPILE_FLAGS "/safeseh"
            )
        endif()
        set_source_files_properties(
            ${cryptopp_SOURCES_ASM}
            PROPERTIES LANGUAGE ASM_MASM
        )
    endif()
endif()

# ============================================================================
# Compile targets
# ============================================================================

# Note that in this function we iterate over C++ sources files only. We do not
# apply options and definitions to the whole target (which includes ASM) because
# the assembler does not take the same options as the C++ compiler.
function(cryptopp_set_compile_properties)
    set(options ${CRYPTOPP_COMPILE_OPTIONS})
    if(MSVC)
        list(APPEND options ${CRYPTOPP_MSVC_COMPILE_OPTIONS})
    endif()
    if(options)
        list(REMOVE_DUPLICATES options)
        if(CMAKE_CXX_FLAGS)
            string(REPLACE " " ";" global_flags ${CMAKE_CXX_FLAGS})
            list(REMOVE_ITEM options ${global_flags})
        endif()
    endif()
    foreach(cxx_file ${cryptopp_SOURCES} ${cryptopp_SOURCES_TEST})
        set_property(
            SOURCE ${cxx_file}
            APPEND
            PROPERTY COMPILE_OPTIONS ${options}
        )
        set_property(
            SOURCE ${cxx_file}
            APPEND
            PROPERTY COMPILE_DEFINITIONS ${CRYPTOPP_COMPILE_DEFINITIONS}
        )
    endforeach()
endfunction()

# Set compiler options and compiler definitions for each CXX source file.
cryptopp_set_compile_properties()

# FIXME For now crypto++ is not written to properly export symbols. This is a
# library problem, and the old DLL was a FIPS only DLL that does not contain
# everything.
#
# The recommended way to use a DLL is to make a wrapper DLL that links
# statically to crypto++.
#
# see https://cryptopp.com/wiki/Wrapper_DLL
#
# This CMakeLists is howeber written to support both shared and static builds.
# We block the shared build at the parent scope.

# The lib we are building should respect and honor the cmake BUILD_SHARED_LIB
# and the rule that requires the build interface to be consistent with the
# install interface. We implement that by having our own CRYPTOPP_BUILD_SHARED
# options that we use to override the BUILD_SHARED_LIBS in our package scope.
if(DEFINED CRYPTOPP_BUILD_SHARED)
    set(BUILD_SHARED_LIBS ${CRYPTOPP_BUILD_SHARED})
endif()

if(
    NOT DEFINED CMAKE_CXX_VISIBILITY_PRESET
    AND NOT DEFINED CMAKE_VISIBILITY_INLINES_HIDDEN
)
    set(CMAKE_CXX_VISIBILITY_PRESET hidden)
    set(CMAKE_VISIBILITY_INLINES_HIDDEN YES)
endif()

# From now on, we only build one type of library per invocation of cmake, shared
# or static, but never both. Which one we build depends on whether
# CRYPTOPP_BUILD_SHARED was ON or OFF.

# Build the complete list of library sources, including CXX, ASM and
# intermediary object files.
set(cryptopp_LIBRARY_SOURCES ${cryptopp_SOURCES_ASM})
list(APPEND cryptopp_LIBRARY_SOURCES ${cryptopp_SOURCES})

add_library(cryptopp ${cryptopp_LIBRARY_SOURCES})
add_library(cryptopp::cryptopp ALIAS cryptopp)
set_target_properties(
    cryptopp
    PROPERTIES
        VERSION ${cryptopp-cmake_VERSION}
        SOVERSION ${cryptopp-cmake_VERSION_MAJOR}
)
set_target_properties(cryptopp PROPERTIES LINKER_LANGUAGE CXX)
if(${CRYPTOPP_BUILD_SHARED})
    target_compile_definitions(cryptopp PRIVATE "CRYPTOPP_EXPORTS")
endif()
target_compile_definitions(
    cryptopp
    INTERFACE
        $<INSTALL_INTERFACE:CRYPTOPP_INCLUDE_PREFIX=${CRYPTOPP_INCLUDE_PREFIX}>
)
cmake_path(GET cryptopp_SOURCE_DIR PARENT_PATH CRYPTOPP_PREFIXED_INCLUDE_DIR)
target_include_directories(
    cryptopp
    PUBLIC
        $<BUILD_INTERFACE:${cryptopp_SOURCE_DIR}>
        $<BUILD_INTERFACE:${CRYPTOPP_PREFIXED_INCLUDE_DIR}>
        $<INSTALL_INTERFACE:include>
)

if(CMAKE_CXX_COMPILER_ID STREQUAL "SunPro")
    target_link_options(cryptopp PRIVATE ${CRYPTOPP_SUN_LDFLAGS})
endif()

# ============================================================================
# Third-party libraries
# ============================================================================

# CMake links to a lot of libraries we don't need in Windows, like user32.lib
# gdi32.lib winspool.lib shell32.lib ole32.lib oleaut32.lib uuid.lib
# comdlg32.lib
if(WIN32)
    target_link_libraries(cryptopp kernel32)
endif()

find_package(Threads)
target_link_libraries(cryptopp ${CMAKE_THREAD_LIBS_INIT})

# ============================================================================
# Setup OpenMP
# ============================================================================
if(CRYPTOPP_USE_OPENMP)
    list(APPEND CMAKE_PREFIX_PATH /usr/local/opt/libomp)
    list(APPEND CMAKE_PREFIX_PATH /opt/homebrew/opt/libomp/)
    list(APPEND CMAKE_PREFIX_PATH /opt/local/lib/libomp/)
    list(APPEND CMAKE_INCLUDE_PATH /opt/local/include/libomp/)
    find_package(OpenMP REQUIRED)
endif()

target_link_libraries(
    cryptopp
    $<$<BOOL:${CRYPTOPP_USE_OPENMP}>:OpenMP::OpenMP_CXX>
)
# ============================================================================
# Tests
# ============================================================================

if(CRYPTOPP_BUILD_TESTING)
    add_executable(cryptest ${cryptopp_SOURCES_TEST})
    target_link_libraries(cryptest PRIVATE cryptopp::cryptopp)

    set_source_files_properties(
        ${cryptopp_SOURCE_DIR}/bench3.cpp
        ${cryptopp_SOURCE_DIR}/datatest.cpp
        ${cryptopp_SOURCE_DIR}/test.cpp
        ${cryptopp_SOURCE_DIR}/validat0.cpp
        ${cryptopp_SOURCE_DIR}/validat4.cpp
        ${cryptopp_SOURCE_DIR}/validat7.cpp
        ${cryptopp_SOURCE_DIR}/validat8.cpp
        ${cryptopp_SOURCE_DIR}/validat9.cpp
        PROPERTIES
            COMPILE_DEFINITIONS
                CRYPTOPP_DATA_DIR="${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_DATAROOTDIR}/cryptopp/"
    )

    add_test(
        NAME cryptopp-build_cryptest
        COMMAND
            "${CMAKE_COMMAND}" --build ${CMAKE_BINARY_DIR} --target cryptest
            --config ${CMAKE_BUILD_TYPE}
    )
    set_tests_properties(
        cryptopp-build_cryptest
        PROPERTIES
            FIXTURES_SETUP cryptest-build
            LABELS "cryptopp;cryptopp-cryptest"
    )

    add_test(
        NAME cryptopp-cryptest
        COMMAND $<TARGET_FILE:cryptest> v
        WORKING_DIRECTORY ${cryptopp_SOURCE_DIR}
    )
    set_tests_properties(
        cryptopp-cryptest
        PROPERTIES
            FIXTURES_REQUIRED cryptest-build
            LABELS "cryptopp;cryptopp-cryptest"
    )

    add_test(
        NAME cryptopp-cryptest-extensive
        COMMAND $<TARGET_FILE:cryptest> tv all
        WORKING_DIRECTORY ${cryptopp_SOURCE_DIR}
    )
    set_tests_properties(
        cryptopp-cryptest-extensive
        PROPERTIES
            FIXTURES_CLEANUP cryptest-build
            LABELS "cryptopp;cryptopp-cryptest"
    )
endif()

# ============================================================================
# Doxygen documentation
# ============================================================================

if(CRYPTOPP_BUILD_DOCUMENTATION)
    find_package(Doxygen REQUIRED)

    set(in_source_DOCS_DIR "${cryptopp_SOURCE_DIR}/html-docs")
    set(out_source_DOCS_DIR "${PROJECT_BINARY_DIR}/html-docs")

    add_custom_target(
        docs
        ALL
        COMMAND ${DOXYGEN_EXECUTABLE} Doxyfile -d CRYPTOPP_DOXYGEN_PROCESSING
        WORKING_DIRECTORY ${cryptopp_SOURCE_DIR}
        SOURCES ${cryptopp_SOURCE_DIR}/Doxyfile
    )

    if(NOT ${in_source_DOCS_DIR} STREQUAL ${out_source_DOCS_DIR})
        add_custom_command(
            TARGET docs
            POST_BUILD
            COMMAND
                ${CMAKE_COMMAND} -E copy_directory "${in_source_DOCS_DIR}"
                "${out_source_DOCS_DIR}"
            COMMAND ${CMAKE_COMMAND} -E remove_directory "${in_source_DOCS_DIR}"
        )
    endif()
endif()

# ==============================================================================
# Deployment instructions
# ==============================================================================

include(GNUInstallDirs)

if(CRYPTOPP_INSTALL)
    set(TARGETS_EXPORT_NAME "cryptopp_Targets")
    set(runtime "cryptopp_runtime")
    set(dev "cryptopp_dev")

    include(ConfigFiles)
    create_module_config_files()

    install(
        TARGETS cryptopp
        EXPORT ${TARGETS_EXPORT_NAME}
        RUNTIME
        COMPONENT ${runtime}
        LIBRARY
        COMPONENT ${runtime}
        ARCHIVE
        COMPONENT ${dev}
    )

    # Header files
    install(
        FILES ${cryptopp_HEADERS}
        COMPONENT ${dev}
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/${CRYPTOPP_INCLUDE_PREFIX}"
    )

    if(CRYPTOPP_BUILD_SHARED)
        set(type shared)
    else()
        set(type static)
    endif()

    install(
        EXPORT ${TARGETS_EXPORT_NAME}
        DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/cmake/cryptopp"
        NAMESPACE cryptopp::
        FILE cryptopp-${type}-targets.cmake
        COMPONENT ${dev}
    )

    # Package configuration files
    install(
        FILES
            ${CMAKE_CURRENT_SOURCE_DIR}/cryptoppConfig.cmake
            ${CMAKE_CURRENT_BINARY_DIR}/cryptoppConfigVersion.cmake
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/cmake/cryptopp
        COMPONENT ${dev}
    )
    install(
        FILES ${CMAKE_CURRENT_BINARY_DIR}/cryptopp.pc
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/pkgconfig
        COMPONENT ${dev}
    )

    # Tests
    if(CRYPTOPP_BUILD_TESTING)
        install(TARGETS cryptest DESTINATION ${CMAKE_INSTALL_BINDIR})
        install(
            DIRECTORY ${cryptopp_SOURCE_DIR}/TestData
            DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/cryptopp
        )
        install(
            DIRECTORY ${cryptopp_SOURCE_DIR}/TestVectors
            DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/cryptopp
        )
    endif()

    # Documentation
    if(CRYPTOPP_BUILD_DOCUMENTATION)
        install(
            DIRECTORY "${out_source_DOCS_DIR}"
            DESTINATION ${CMAKE_INSTALL_DOCDIR}
        )
    endif()
endif()

# Print a configuration summary. We want CXX and CXXFLAGS, but they are not
# includd in ALL.
if(CRYPTOPP_I386)
    message(STATUS "[cryptopp] Platform: i386/i686")
elseif(CRYPTOPP_AMD64)
    message(STATUS "[cryptopp] Platform: x86_64")
elseif(CRYPTOPP_ARM32)
    message(STATUS "[cryptopp] Platform: ARM-32")
elseif(CRYPTOPP_ARMV8)
    message(STATUS "[cryptopp] Platform: ARMv8")
elseif(CRYPTOPP_SPARC)
    message(STATUS "[cryptopp] Platform: Sparc")
elseif(CRYPTOPP_SPARC64)
    message(STATUS "[cryptopp] Platform: Sparc64")
elseif(CRYPTOPP_PPC32)
    message(STATUS "[cryptopp] Platform: PowerPC")
elseif(CRYPTOPP_PPC64)
    message(STATUS "[cryptopp] Platform: PowerPC-64")
elseif(CRYPTOPP_MINGW32)
    message(STATUS "[cryptopp] Platform: MinGW-32")
elseif(CRYPTOPP_MINGW64)
    message(STATUS "[cryptopp] Platform: MinGW-64")
endif()
if(CRYPTOPP_HAVE_ARM_NEON)
    message(STATUS "[cryptopp] NEON: TRUE")
endif()
message(
    STATUS
    "[cryptopp] Compiler definitions: ${CMAKE_CPP_FLAGS} ${CRYPTOPP_COMPILE_DEFINITIONS}"
)
message(
    STATUS
    "[cryptopp] Compiler options: ${CMAKE_CXX_FLAGS} ${CRYPTOPP_COMPILE_OPTIONS} ${CRYPTOPP_MSVC_COMPILE_OPTIONS}"
)
message(STATUS "[cryptopp] Build type: ${CMAKE_BUILD_TYPE}")
