#include "backtrace.h"
#include <execinfo.h>
#include <signal.h>
#include <iostream>
#include <unistd.h>
#include <cxxabi.h>
#include <string>

using std::string;

//TODO Use the following? https://github.com/bombela/backward-cpp

namespace cpputils {

    //TODO Refactor (for example: RAII or at least try{}finally{} instead of not-exceptionsafe free())

    std::string demangle(const string &mangledName) {
        string result;
        int status = -10;
        char *demangledName = abi::__cxa_demangle(mangledName.c_str(), NULL, NULL, &status);
        if (status == 0) {
            result = demangledName;
        } else {
            result = mangledName;
        }
        free(demangledName);
        return result;
    }

    std::string pretty(const string &backtraceLine) {
        size_t startMangledName = backtraceLine.find('(');
        size_t endMangledName = backtraceLine.find('+');
        if (startMangledName == string::npos || endMangledName == string::npos) {
            return backtraceLine;
        }
        return demangle(backtraceLine.substr(startMangledName+1, endMangledName-startMangledName-1)) + ": (" + backtraceLine.substr(0, startMangledName) + backtraceLine.substr(endMangledName);
    }

    void print_backtrace(void *array[], size_t size) {
        char **ptr = backtrace_symbols(array, size);
        for (size_t i = 0; i < size; ++i) {
            std::cerr << pretty(ptr[i]) << "\n";
        }
        free(ptr);
    }

    void sigsegv_handler(int) {
        constexpr unsigned int MAX_SIZE = 100;
        void *array[MAX_SIZE];
        size_t size = backtrace(array, MAX_SIZE);

        std::cerr << "Error: SIGSEGV" << std::endl;
        print_backtrace(array, size);
        exit(1);
    }

    void showBacktraceOnSigSegv() {
        signal(SIGSEGV, sigsegv_handler);
    }
}
