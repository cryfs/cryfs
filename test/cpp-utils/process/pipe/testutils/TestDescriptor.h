#pragma once
#ifndef MESSMER_CPPUTILS_TEST_PROCESS_PIPE_TESTUTILS_TESTDESCRIPTOR_H
#define MESSMER_CPPUTILS_TEST_PROCESS_PIPE_TESTUTILS_TESTDESCRIPTOR_H

#include <fcntl.h>

class TestDescriptor final {
public:
    TestDescriptor() {
        int fds[2];
        EXPECT_EQ(0, ::pipe(fds));
        ::close(fds[0]);
        _fd = fds[1];
    }

    ~TestDescriptor() {
        // If already closed, this will return an error. So ignore return value, this is just meant for cleanup.
        ::close(_fd);
    }

    int get() const {
        return _fd;
    }

private:
    int _fd;

    DISALLOW_COPY_AND_ASSIGN(TestDescriptor);
};

#endif
