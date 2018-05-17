#if !defined(_MSC_VER)

#include "backtrace.h"
#include <execinfo.h>
#include <signal.h>
#include <iostream>
#include <unistd.h>
#include <cxxabi.h>
#include <string>
#include <sstream>
#include "../logging/logging.h"

using std::string;
using std::ostringstream;
using namespace cpputils::logging;

namespace cpputils {

namespace {
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
        return demangle(backtraceLine.substr(startMangledName + 1, endMangledName - startMangledName - 1)) + ": (" +
               backtraceLine.substr(0, startMangledName) + backtraceLine.substr(endMangledName);
    }

    string backtrace_to_string(void *array[], size_t size) {
        ostringstream result;
        char **ptr = backtrace_symbols(array, size);
        for (size_t i = 0; i < size; ++i) {
            result << pretty(ptr[i]) << "\n";
        }
        free(ptr);
        return result.str();
    }
}

	string backtrace() {
		constexpr unsigned int MAX_SIZE = 100;
		void *array[MAX_SIZE];
		size_t size = ::backtrace(array, MAX_SIZE);
		return backtrace_to_string(array, size);
	}

namespace {
    void sigsegv_handler(int) {
        LOG(ERROR, "SIGSEGV\n{}", backtrace());
        exit(1);
    }
}

	void showBacktraceOnSigSegv() {
		auto result = signal(SIGSEGV, sigsegv_handler);
		if (SIG_ERR == result) {
			LOG(ERROR, "Failed to set sigsegv signal handler. Errno: {}", errno);
		}
	}

}

#endif
