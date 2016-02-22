project (cpp-utils-test)

set(SOURCES
    EitherIncludeTest.cpp
    crypto/symmetric/CipherTest.cpp
    crypto/symmetric/testutils/FakeAuthenticatedCipher.cpp
    crypto/kdf/SCryptTest.cpp
    crypto/kdf/SCryptParametersTest.cpp
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
    tempfile/TempFileTest.cpp
    tempfile/TempFileIncludeTest.cpp
    tempfile/TempDirIncludeTest.cpp
    tempfile/TempDirTest.cpp
    network/CurlHttpClientTest.cpp
    network/FakeHttpClientTest.cpp
    io/ConsoleIncludeTest.cpp
    io/ConsoleTest_AskYesNo.cpp
    io/ConsoleTest_Print.cpp
    io/ConsoleTest_Ask.cpp
    random/RandomIncludeTest.cpp
    lock/LockPoolIncludeTest.cpp
    lock/ConditionBarrierIncludeTest.cpp
    lock/MutexPoolLockIncludeTest.cpp
    data/FixedSizeDataTest.cpp
    data/DataFixtureIncludeTest.cpp
    data/DataFixtureTest.cpp
    data/DataTest.cpp
    data/FixedSizeDataIncludeTest.cpp
    data/DataIncludeTest.cpp
    logging/LoggingLevelTest.cpp
    logging/LoggerTest.cpp
    logging/LoggingTest.cpp
    logging/LoggerIncludeTest.cpp
    logging/LoggingIncludeTest.cpp
    EitherTest.cpp
    assert/assert_release_test.cpp
    assert/backtrace_include_test.cpp
    assert/assert_include_test.cpp
    assert/assert_debug_test.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})
target_link_libraries(${PROJECT_NAME} googletest cpp-utils)
add_test(${PROJECT_NAME} ${PROJECT_NAME})

target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
