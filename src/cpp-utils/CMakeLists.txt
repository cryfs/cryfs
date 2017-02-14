project (cpp-utils)

set(SOURCES
        crypto/symmetric/ciphers.cpp
        crypto/kdf/Scrypt.cpp
        crypto/kdf/SCryptParameters.cpp
        crypto/kdf/PasswordBasedKDF.cpp
        crypto/RandomPadding.cpp
        process/pipe/PipeBuilder.cpp
        process/pipe/PipeDescriptor.cpp
        process/pipe/PipeNotReadableError.cpp
        process/pipe/PipeReader.cpp
        process/pipe/PipeStreamEndpoint.cpp
        process/pipe/PipeWriter.cpp
        process/daemon/daemonize.cpp
        process/daemon/Messages.cpp
        process/daemon/PipeFromChild.cpp
        process/daemon/PipeToParent.cpp
        process/subprocess.cpp
        tempfile/TempFile.cpp
        tempfile/TempDir.cpp
        network/HttpClient.cpp
        network/CurlHttpClient.cpp
        network/CurlInitializerRAII.cpp
        network/FakeHttpClient.cpp
        io/Console.cpp
        io/DontEchoStdinToStdoutRAII.cpp
        io/IOStreamConsole.cpp
        io/NoninteractiveConsole.cpp
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

# Needed so it can be linked in to the shared cryfs library
set_target_properties(${PROJECT_NAME} PROPERTIES POSITION_INDEPENDENT_CODE on)
