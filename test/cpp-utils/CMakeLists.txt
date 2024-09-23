project (cpp-utils-test)

set(SOURCES
    crypto/symmetric/CipherTest.cpp
    crypto/kdf/SCryptTest.cpp
    crypto/kdf/SCryptParametersTest.cpp
    crypto/hash/HashTest.cpp
    MacrosIncludeTest.cpp
    pointer/unique_ref_test.cpp
    pointer/cast_include_test.cpp
    pointer/cast_test.cpp
    pointer/unique_ref_boost_optional_gtest_workaround_include_test.cpp
    pointer/optional_ownership_ptr_include_test.cpp
    pointer/optional_ownership_ptr_test.cpp
    pointer/unique_ref_include_test.cpp
    process/daemonize_include_test.cpp
    process/subprocess_include_test.cpp
    process/SubprocessTest.cpp
	process/SignalCatcherTest.cpp
	process/SignalHandlerTest.cpp
    tempfile/TempFileTest.cpp
    tempfile/TempFileIncludeTest.cpp
    tempfile/TempDirIncludeTest.cpp
    tempfile/TempDirTest.cpp
    network/CurlHttpClientTest.cpp
    network/FakeHttpClientTest.cpp
    io/DontEchoStdinToStdoutRAIITest.cpp
    io/ConsoleIncludeTest.cpp
    io/ConsoleTest_AskYesNo.cpp
    io/ConsoleTest_Print.cpp
    io/ConsoleTest_Ask.cpp
	io/ConsoleTest_AskPassword.cpp
	io/ProgressBarTest.cpp
    random/RandomIncludeTest.cpp
    lock/LockPoolIncludeTest.cpp
    lock/ConditionBarrierIncludeTest.cpp
    lock/MutexPoolLockIncludeTest.cpp
    data/FixedSizeDataTest.cpp
    data/DataFixtureIncludeTest.cpp
    data/DataFixtureTest.cpp
    data/DataTest.cpp
    data/FixedSizeDataIncludeTest.cpp
    data/SerializationHelperTest.cpp
    data/DataIncludeTest.cpp
    logging/LoggingLevelTest.cpp
    logging/LoggerTest.cpp
    logging/LoggingTest.cpp
    logging/LoggerIncludeTest.cpp
    logging/LoggingIncludeTest.cpp
    assert/assert_release_test.cpp
    assert/backtrace_test.cpp
    assert/assert_debug_test.cpp
    system/GetTotalMemoryTest.cpp
    system/TimeTest.cpp
	system/PathTest.cpp
	system/FiletimeTest.cpp
    system/MemoryTest.cpp
    system/HomedirTest.cpp
	system/EnvTest.cpp
	thread/debugging_test.cpp
	thread/LeftRightTest.cpp
    value_type/ValueTypeTest.cpp
	either_test.cpp
)

add_executable(${PROJECT_NAME}_exit_status process/exit_status.cpp)
target_activate_cpp14(${PROJECT_NAME}_exit_status)

add_executable(${PROJECT_NAME}_exit_signal assert/exit_signal.cpp)
target_activate_cpp14(${PROJECT_NAME}_exit_signal)
target_link_libraries(${PROJECT_NAME}_exit_signal cpp-utils)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} my-gtest-main cpp-utils)
add_dependencies(${PROJECT_NAME} ${PROJECT_NAME}_exit_status ${PROJECT_NAME}_exit_signal)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
