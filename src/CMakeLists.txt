include(CheckCXXCompilerFlag)

###################################################
#  Activate C++14
#
#  Uses: ACTIVATE_CPP14()
###################################################
macro(ACTIVATE_CPP14)
  CHECK_CXX_COMPILER_FLAG("-std=c++14" COMPILER_HAS_CPP14_SUPPORT)
  IF (COMPILER_HAS_CPP14_SUPPORT)
    TARGET_COMPILE_OPTIONS(${PROJECT_NAME} PUBLIC -std=c++14)
  ELSE()
    CHECK_CXX_COMPILER_FLAG("-std=c++1y" COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
    IF (COMPILER_HAS_CPP14_PARTIAL_SUPPORT)
      TARGET_COMPILE_OPTIONS(${PROJECT_NAME} PUBLIC -std=c++1y)
    ELSE()
      MESSAGE(FATAL_ERROR "Compiler doesn't support C++14")
    ENDIF()
  ENDIF()
  IF(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    TARGET_COMPILE_OPTIONS(${PROJECT_NAME} PUBLIC -stdlib=libc++)
  ENDIF()
endmacro(ACTIVATE_CPP14)

#################################################
# Enable style compiler warnings
#################################################
macro(ENABLE_STYLE_WARNINGS)
#  TARGET_COMPILE_OPTIONS(${BII_BLOCK_TARGET} INTERFACE -Wall -Wextra -Weffc++)
  TARGET_COMPILE_OPTIONS(${PROJECT_NAME} PRIVATE -Wall -Wextra)
endmacro(ENABLE_STYLE_WARNINGS)

##################################################
# Add boost to the project
#
# Uses:
#  ADD_BOOST() # if you're only using header-only boost libs
#  ADD_BOOST(system filesystem) # list all libraries to link against in the dependencies
##################################################
function(ADD_BOOST)
  # Load boost libraries
  find_package(Boost 1.56.0
               REQUIRED
               COMPONENTS ${ARGN})
  set(Boost_USE_STATIC_LIBS ON)
  target_include_directories(${PROJECT_NAME} SYSTEM PRIVATE ${Boost_INCLUDE_DIRS})
  target_link_libraries(${PROJECT_NAME} PRIVATE ${Boost_LIBRARIES})
endfunction()

cmake_minimum_required(VERSION 3.0)
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
  TARGET_LINK_LIBRARIES(${PROJECT_NAME} PRIVATE rt)
endif(${CMAKE_SYSTEM_NAME} MATCHES "Linux")

TARGET_LINK_LIBRARIES(${PROJECT_NAME} PRIVATE curl)

# TODO From Crypto++ 5.7 on, it should support cmake with find_package().
find_library(CryptoPP cryptopp)
target_link_libraries(${PROJECT_NAME} PRIVATE ${CryptoPP} scrypt)

ADD_BOOST(filesystem system thread)
ENABLE_STYLE_WARNINGS()
ACTIVATE_CPP14()
