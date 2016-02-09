include(ExternalProject)

add_subdirectory(gtest-1.7.0)

project (scrypt C)
# This builds the original scrypt source using configure & make. We don't use their build result,
# but we use the config.h it creates somewhere on the way (see target_compile_options below).
ExternalProject_Add(scrypt_original
        SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0
        CONFIGURE_COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/configure --prefix=${CMAKE_BINARY_DIR}/vendor/
        PREFIX ${CMAKE_BINARY_DIR}/vendor/
        BUILD_COMMAND ${MAKE})
# This creates the cpusupport_config.h file we need (see target_compile_options below).
execute_process(
        COMMAND /bin/sh ${CMAKE_CURRENT_LIST_DIR}/scrypt-1.2.0/libcperciva/cpusupport/Build/cpusupport.sh
        OUTPUT_FILE ${CMAKE_CURRENT_BINARY_DIR}/cpusupport_config.h
        RESULT_VARIABLE result
)
add_library(${PROJECT_NAME} STATIC
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/lib/crypto/crypto_scrypt.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/lib/crypto/crypto_scrypt_smix.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/lib/crypto/crypto_scrypt_smix_sse2.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/util/warnp.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/util/insecure_memzero.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/alg/sha256.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/cpusupport/cpusupport_x86_aesni.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/cpusupport/cpusupport_x86_sse2.c
)
target_include_directories(${PROJECT_NAME} PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/cpusupport
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/alg
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/libcperciva/util
)
target_compile_options(${PROJECT_NAME} PRIVATE
        -DCONFIG_H_FILE=\"${CMAKE_CURRENT_BINARY_DIR}/src/scrypt_original-build/config.h\"
        -DCPUSUPPORT_CONFIG_FILE=\"${CMAKE_CURRENT_BINARY_DIR}/cpusupport_config.h\"
)
add_dependencies(${PROJECT_NAME} scrypt_original)

