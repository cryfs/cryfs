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
        assert/assert.cpp
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

if(MSVC)
    target_link_libraries(${PROJECT_NAME} PUBLIC DbgHelp)
elseif (APPLE)
    target_compile_definitions(${PROJECT_NAME} PRIVATE BOOST_STACKTRACE_GNU_SOURCE_NOT_REQUIRED)
else()
    find_program(ADDR2LINE addr2line)
    if ("${ADDR2LINE}" STREQUAL "ADDR2LINE-NOTFOUND")
        message(WARNING "addr2line not found. Backtraces will be reduced.")
    else()
        message(STATUS "addr2line found. Using it for backtraces.")
        target_compile_definitions(${PROJECT_NAME} PRIVATE BOOST_STACKTRACE_USE_ADDR2LINE)
        target_compile_definitions(${PROJECT_NAME} PRIVATE BOOST_STACKTRACE_ADDR2LINE_LOCATION=${ADDR2LINE})
    endif()
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

target_link_libraries(${PROJECT_NAME} PUBLIC CryfsDependencies_spdlog cryptopp CryfsDependencies_range-v3)

target_add_boost(${PROJECT_NAME} filesystem system thread chrono)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

if(MSVC)
    # Required by range-v3, see its README.md
    target_compile_options(${PROJECT_NAME} PUBLIC /experimental:preprocessor /permissive- /Zc:twoPhase-)
endif()
