#include "backtrace.h"
#include <execinfo.h>
#include <signal.h>
#include <iostream>
#include <unistd.h>

//TODO Use the following? https://github.com/bombela/backward-cpp

namespace cpputils {

    void sigsegv_handler(int) {
        constexpr unsigned int MAX_SIZE = 100;
        void *array[MAX_SIZE];
        size_t size = backtrace(array, MAX_SIZE);

        std::cerr << "Error: SIGSEGV" << std::endl;
        backtrace_symbols_fd(array, size, STDERR_FILENO);
        exit(1);
    }

    void showBacktraceOnSigSegv() {
        signal(SIGSEGV, sigsegv_handler);
    }
}
