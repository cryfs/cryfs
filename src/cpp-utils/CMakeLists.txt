include(CheckCXXCompilerFlag)

project (cpp-utils)

set(SOURCES
        crypto/symmetric/ciphers.cpp
        crypto/kdf/DerivedKey.cpp
        crypto/kdf/Scrypt.cpp
        crypto/kdf/DerivedKeyConfig.cpp
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
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

# This is needed by boost thread
if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
  target_link_libraries(${PROJECT_NAME} PRIVATE rt)
endif(${CMAKE_SYSTEM_NAME} MATCHES "Linux")

target_link_libraries(${PROJECT_NAME} PRIVATE curl)

# TODO From Crypto++ 5.7 on, it should support cmake with find_package().
find_library(CryptoPP cryptopp)
target_link_libraries(${PROJECT_NAME} PRIVATE ${CryptoPP} scrypt)

add_boost(${PROJECT_NAME} filesystem system thread)
enable_style_warnings(${PROJECT_NAME})
activate_cpp14(${PROJECT_NAME})
