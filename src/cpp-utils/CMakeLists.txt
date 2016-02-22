project (cpp-utils)

set(SOURCES
        crypto/symmetric/ciphers.cpp
        crypto/kdf/Scrypt.cpp
        crypto/kdf/SCryptParameters.cpp
        crypto/kdf/PasswordBasedKDF.cpp
        crypto/RandomPadding.cpp
        process/daemonize.cpp
        process/subprocess.cpp
        tempfile/TempFile.cpp
        tempfile/TempDir.cpp
        network/HttpClient.cpp
        network/CurlHttpClient.cpp
        network/FakeHttpClient.cpp
        io/Console.cpp
        io/pipestream.cpp
        thread/LoopThread.cpp
        thread/ThreadSystem.cpp
        random/Random.cpp
        random/RandomGeneratorThread.cpp
        random/OSRandomGenerator.cpp
        random/PseudoRandomPool.cpp
        random/RandomDataBuffer.cpp
        random/RandomGenerator.cpp
        lock/LockPool.cpp
        data/Serializer.cpp
        data/Deserializer.cpp
        data/DataFixture.cpp
        data/DataUtils.cpp
        data/Data.cpp
        assert/backtrace.cpp
        assert/AssertFailed.cpp
        system/get_total_memory.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC pthread curl)

# TODO From Crypto++ 5.7 on, it should support cmake with find_package() instead of find_library().
find_library_with_path(CryptoPP cryptopp CRYPTOPP_LIB_PATH)
target_link_libraries(${PROJECT_NAME} PUBLIC ${CryptoPP} scrypt spdlog)

target_add_boost(${PROJECT_NAME} filesystem system thread)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
