#pragma once
#ifndef MESSMER_CRYFS_TEST_TESTUTILS_TESTWITHFAKEHOMEDIRECTORY_H
#define MESSMER_CRYFS_TEST_TESTUTILS_TESTWITHFAKEHOMEDIRECTORY_H

#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/system/homedir.h>

class TestWithFakeHomeDirectory {
public:
    TestWithFakeHomeDirectory()
        :homedir(), fakeHomeDirRAII(homedir.path()) {
    }

private:
    cpputils::TempDir homedir;
    cpputils::system::FakeHomeDirectoryRAII fakeHomeDirRAII;
};

#endif
