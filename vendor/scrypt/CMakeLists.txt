include(ExternalProject)

project (scrypt C)

# This builds the original scrypt source using configure & make. We don't use their build result,
# but we use the config.h it creates somewhere on the way (see target_compile_options below).
string(TOUPPER "${CMAKE_BUILD_TYPE}" BUILDTYPE)
set(SCRYPT_CFLAGS "${CMAKE_C_FLAGS} ${CMAKE_C_FLAGS_${BUILDTYPE}}")

# Add a default location for homebrew-installed openssl on Mac OS X to include search path
if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(SCRYPT_CFLAGS "${SCRYPT_CFLAGS} -I/usr/local/opt/openssl/include")
endif(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")

ExternalProject_Add(scrypt_original
        SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt
        CONFIGURE_COMMAND /usr/bin/env CC=${CMAKE_C_COMPILER} CFLAGS=${SCRYPT_CFLAGS} ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/configure --prefix=${CMAKE_BINARY_DIR}/vendor/scrypt/
        PREFIX ${CMAKE_BINARY_DIR}/vendor/scrypt/
        BUILD_COMMAND ${MAKE}
)
execute_process(
        COMMAND /bin/sh ${CMAKE_CURRENT_LIST_DIR}/scrypt-1.2.0/scrypt/libcperciva/cpusupport/Build/cpusupport.sh
        OUTPUT_FILE ${CMAKE_CURRENT_BINARY_DIR}/cpusupport_config.h
        RESULT_VARIABLE result
)
add_library(${PROJECT_NAME} STATIC
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/lib/crypto/crypto_scrypt.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/lib/crypto/crypto_scrypt_smix.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/util/warnp.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/util/insecure_memzero.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/alg/sha256.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/cpusupport/cpusupport_x86_aesni.c
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/cpusupport/cpusupport_x86_sse2.c
)
target_link_libraries(${PROJECT_NAME}
        ${CMAKE_CURRENT_BINARY_DIR}/src/scrypt_original-build/libscrypt_sse2.a
        ${CMAKE_CURRENT_BINARY_DIR}/src/scrypt_original-build/libcperciva_aesni.a
)
target_include_directories(${PROJECT_NAME} PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/cpusupport
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/alg
        ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0/scrypt/libcperciva/util
)
target_compile_options(${PROJECT_NAME} PRIVATE
        -DCONFIG_H_FILE=\"${CMAKE_CURRENT_BINARY_DIR}/src/scrypt_original-build/config.h\"
        -DCPUSUPPORT_CONFIG_FILE=\"${CMAKE_CURRENT_BINARY_DIR}/cpusupport_config.h\"
)
add_dependencies(${PROJECT_NAME} scrypt_original)
target_include_directories(${PROJECT_NAME} SYSTEM INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/scrypt-1.2.0)
