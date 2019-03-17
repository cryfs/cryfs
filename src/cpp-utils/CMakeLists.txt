project (cpp-utils)

set(SOURCES
        crypto/symmetric/ciphers.cpp
        crypto/symmetric/testutils/FakeAuthenticatedCipher.cpp
        crypto/kdf/Scrypt.cpp
        crypto/kdf/SCryptParameters.cpp
        crypto/kdf/PasswordBasedKDF.cpp
        crypto/RandomPadding.cpp
        crypto/symmetric/EncryptionKey.cpp
        crypto/hash/Hash.cpp
        process/daemonize.cpp
        process/subprocess.cpp
        process/SignalCatcher.cpp
        process/SignalHandler.cpp
        tempfile/TempFile.cpp
        tempfile/TempDir.cpp
        network/HttpClient.cpp
        network/CurlHttpClient.cpp
		network/WinHttpClient.cpp
        network/FakeHttpClient.cpp
        io/Console.cpp
        io/DontEchoStdinToStdoutRAII.cpp
        io/IOStreamConsole.cpp
        io/NoninteractiveConsole.cpp
        io/pipestream.cpp
        io/ProgressBar.cpp
        thread/LoopThread.cpp
        thread/ThreadSystem.cpp
        thread/debugging_nonwindows.cpp
        thread/debugging_windows.cpp
        thread/LeftRight.cpp
        random/Random.cpp
        random/RandomGeneratorThread.cpp
        random/OSRandomGenerator.cpp
        random/PseudoRandomPool.cpp
        random/RandomDataBuffer.cpp
        random/RandomGenerator.cpp
        lock/LockPool.cpp
        data/SerializationHelper.cpp
        data/Serializer.cpp
        data/Deserializer.cpp
        data/DataFixture.cpp
        data/DataUtils.cpp
        data/Data.cpp
        assert/assert.h
        assert/backtrace_nonwindows.cpp
        assert/backtrace_windows.cpp
        assert/AssertFailed.cpp
        system/get_total_memory.cpp
        system/homedir.cpp
        system/memory_nonwindows.cpp
        system/memory_windows.cpp
        system/time.cpp
		system/diskspace.cpp
		system/filetime_nonwindows.cpp
		system/filetime_windows.cpp
		system/env.cpp
        value_type/ValueType.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})


if(NOT MSVC)
  find_package(Backtrace REQUIRED)
  target_include_directories(${PROJECT_NAME} PUBLIC ${Backtrace_INCLUDE_DIRS})
  target_link_libraries(${PROJECT_NAME} PUBLIC ${Backtrace_LIBRARIES})
else()
  target_link_libraries(${PROJECT_NAME} PUBLIC DbgHelp)
endif()

if (NOT MSVC)
  find_package(CURL REQUIRED)
  target_include_directories(${PROJECT_NAME} PUBLIC ${CURL_INCLUDE_DIRS})
  target_link_libraries(${PROJECT_NAME} PUBLIC ${CURL_LIBRARIES})
else()
  target_link_libraries(${PROJECT_NAME} PUBLIC WinHttp)
endif()

find_package(Threads REQUIRED)
target_link_libraries(${PROJECT_NAME} PUBLIC ${CMAKE_THREAD_LIBS_INIT})

target_link_libraries(${PROJECT_NAME} PUBLIC ${CMAKE_DL_LIBS})

target_link_libraries(${PROJECT_NAME} PUBLIC spdlog cryptopp)

target_add_boost(${PROJECT_NAME} filesystem system thread)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
