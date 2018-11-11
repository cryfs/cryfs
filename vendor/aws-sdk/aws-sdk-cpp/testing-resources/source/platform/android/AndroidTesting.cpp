/*
 * Copyright 2010-2017 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License").
 * You may not use this file except in compliance with the License.
 * A copy of the License is located at
 * 
 *  http://aws.amazon.com/apache2.0
 * 
 * or in the "license" file accompanying this file. This file is distributed
 * on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing
 * permissions and limitations under the License.
 */

#include <aws/testing/platform/android/AndroidTesting.h>

#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <android/log.h>
#include <chrono>
#include <thread>
#include <sys/stat.h>
#include <fcntl.h>

#include <aws/external/gtest.h>

#include <aws/core/Aws.h>
#include <aws/core/platform/Platform.h>
#include <aws/core/platform/FileSystem.h>
#include <aws/core/utils/UnreferencedParam.h>
#include <aws/core/utils/memory/stl/AWSString.h>
#include <aws/core/utils/logging/AWSLogging.h>
#include <aws/core/utils/logging/android/LogcatLogSystem.h>
#include <aws/testing/MemoryTesting.h>

#include <jni.h>
#include <iostream>

/*
This redirect solution is based on a blog post found at:
    https://codelab.wordpress.com/2014/11/03/how-to-use-standard-output-streams-for-logging-in-android-apps/

The logging thread function has been reworked substantially to format output correctly.

 */
static int pfd[2];
static pthread_t thr;
static const char *tag = "NativeSDK";

static void *LoggingFunction(void*)
{
  ssize_t rdsz;
  char buf[128];

  std::string logLine;
  while((rdsz = read(pfd[0], buf, sizeof(buf) - 1)) > 0) 
  {
    size_t start = 0;
    size_t current = 0;
    size_t end = rdsz;
    while( current < end )
    {
      if(buf[current] == '\n')
      {
	logLine.append(buf + start, current - start);
	__android_log_write(ANDROID_LOG_DEBUG, tag, logLine.c_str());
	++current;
	start = current;
	logLine = "";
      }
      else
      {
        ++current;
      }
    }

    if( current - start > 0 )
    {
      logLine.append(buf + start, current - start);
    }
  }
  
  return 0;
}

static int StartLogger()
{
    /* make stdout line-buffered and stderr unbuffered */
    setvbuf(stdout, 0, _IOLBF, 0);
    setvbuf(stderr, 0, _IONBF, 0);

    /* create the pipe and redirect stdout and stderr */
    pipe(pfd);
    dup2(pfd[1], 1);
    dup2(pfd[1], 2);

    /* spawn the logging thread */
    if(pthread_create(&thr, 0, LoggingFunction, 0) == -1)
        return -1;
    pthread_detach(thr);
    return 0;
}

void RedirectStdoutToLogcat()
{
  StartLogger();

  std::this_thread::sleep_for(std::chrono::seconds(1));
}

/*

Based on http://stackoverflow.com/questions/2180079/how-can-i-copy-a-file-on-unix-using-c

 */
static int CopyFile(const char *from, const char *to)
{
    int fd_to, fd_from;
    char buf[4096];
    ssize_t nread;
    int saved_errno;

    fd_from = open(from, O_RDONLY);
    if (fd_from < 0)
    {
        return -1;
    }

    fd_to = open(to, O_WRONLY | O_CREAT | O_EXCL, 0666);
    if (fd_to < 0)
    {
        goto out_error;
    }

    while (nread = read(fd_from, buf, sizeof buf), nread > 0)
    {
        char *out_ptr = buf;
        ssize_t nwritten;

        do {
            nwritten = write(fd_to, out_ptr, nread);

            if (nwritten >= 0)
            {
                nread -= nwritten;
                out_ptr += nwritten;
            }
            else if (errno != EINTR)
            {
                goto out_error;
            }
        } while (nread > 0);
    }

    if (nread == 0)
    {
        if (close(fd_to) < 0)
        {
            fd_to = -1;
            goto out_error;
        }
        close(fd_from);

        /* Success! */
        return 0;
    }

  out_error:
    saved_errno = errno;

    close(fd_from);
    if (fd_to >= 0)
        close(fd_to);

    errno = saved_errno;
    return -1;
}

#ifdef __ANDROID__

static const char* ALLOCATION_TAG = "AndroidTests";

#pragma GCC diagnostic ignored "-Wwrite-strings"

static jint RunAndroidTestsInternal()
{
  RedirectStdoutToLogcat();

  std::cout << "Running all enabled Android tests" << std::endl;

  int dummy = 1;
  static char *dummy2 = "Stuff";

  Aws::SDKOptions options;
  Aws::InitAPI(options);

  Aws::Utils::Logging::InitializeAWSLogging(Aws::MakeShared<Aws::Utils::Logging::LogcatLogSystem>(ALLOCATION_TAG, Aws::Utils::Logging::LogLevel::Error));
  ::testing::InitGoogleTest(&dummy, &dummy2);
  auto result = RUN_ALL_TESTS();

  std::this_thread::sleep_for(std::chrono::seconds(3));

  Aws::Utils::Logging::ShutdownAWSLogging();

  Aws::ShutdownAPI(options);

  return (jint) result;
}

// Copes a file that's been uploaded to the activity's directory into the activity's cache, preserving the directory structure
void CacheFile(const Aws::String &fileName, const Aws::String& directory)
{
    Aws::String destDirectory = Aws::Platform::GetCacheDirectory() + directory;
    Aws::FileSystem::CreateDirectoryIfNotExists(destDirectory.c_str());

    Aws::String sourceFileName = "/data/data/aws.androidsdktesting/" + directory + Aws::String( "/" ) + fileName;
    Aws::String destFileName = destDirectory + Aws::String( "/" ) + fileName;

    Aws::String logLine = sourceFileName + " -> " + destFileName;
    __android_log_write(ANDROID_LOG_DEBUG, "Caching ", logLine.c_str());

    CopyFile(sourceFileName.c_str(), destFileName.c_str());
}

static const char* s_SigV4TestNames[] = 
{
    "get-header-key-duplicate",
    "get-header-value-multiline",
    "get-header-value-order",
    "get-header-value-trim",
    "get-relative",
    "get-relative-relative",
    "get-slash",
    "get-slash-dot-slash",
    "get-slash-pointless-dot",
    "get-slashes",
    "get-space",
    "get-unreserved",
    "get-utf8",
    "get-vanilla",
    "get-vanilla-empty-query-key",
    "get-vanilla-query",
    "get-vanilla-query-order-key-case",
    "get-vanilla-query-unreserved",
    "get-vanilla-utf8-query",
    "normalize-path",
    "post-header-key-case",
    "post-header-key-sort",
    "post-header-value-case",
    "post-sts-header-after",
    "post-sts-header-before",
    "post-sts-token",
    "post-vanilla",
    "post-vanilla-empty-query-value",
    "post-vanilla-query",
    "post-vanilla-query-nonunreserved",
    "post-vanilla-query-space",
    "post-x-www-form-urlencoded",
    "post-x-www-form-urlencoded-parameters"
};

static const char* s_SigV4TestSuffixes[] = {
    "authz",
    "creq",
    "req",
    "sreq",
    "sts"
};

void CacheSigV4Tests(const Aws::String& baseDirectory)
{
    uint32_t sigV4TestCount = sizeof(s_SigV4TestNames) / sizeof(s_SigV4TestNames[0]);

    for(uint32_t i = 0; i < sigV4TestCount; ++i)
    {
	Aws::String testName(s_SigV4TestNames[i]);
	Aws::String destDirectory = baseDirectory + Aws::FileSystem::PATH_DELIM + testName;
	
	uint32_t testFileCount = sizeof(s_SigV4TestSuffixes) / sizeof(s_SigV4TestSuffixes[0]);
	for(uint32_t j = 0; j < testFileCount; ++j)
	{
	    Aws::String testFileName = testName + Aws::String(".") + Aws::String(s_SigV4TestSuffixes[j]);
	    CacheFile(testFileName, destDirectory);
	}
    }
}

static const char * s_resourceDirectory = "resources";

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

JNIEXPORT jint JNICALL
Java_aws_androidsdktesting_RunSDKTests_runTests( JNIEnv* env, jobject classRef, jobject context )
{
  Aws::Platform::InitAndroid(env, context);

  // If we upload files to where we expect them to be (cache) then we lose write access to the cache
  // directory since it gets created by the root user before the application has an opportunity to create it.  
  // So when running tests, wait until the application
  // is running before copying data from their upload location to their expected location.
  //
  // Real development should be done via the Cognito/PersistentIdentity credentials providers, where this is not as much
  // a problem
  CacheFile("credentials", ".aws");
  CacheFile("handled.zip", s_resourceDirectory);
  CacheFile("succeed.zip", s_resourceDirectory);
  CacheFile("unhandled.zip", s_resourceDirectory);
  CacheSigV4Tests(s_resourceDirectory);

  jint result = 0;
  AWS_UNREFERENCED_PARAM(classRef);
  AWS_BEGIN_MEMORY_TEST(1024, 128)

  result = RunAndroidTestsInternal();

  AWS_END_MEMORY_OVERRIDE

  return result;
}

#ifdef __cplusplus
}
#endif // __cplusplus

#endif //  __ANDROID__
