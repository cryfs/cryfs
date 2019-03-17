#if !defined(_MSC_VER)

#include "backtrace.h"
#include <execinfo.h>
#include <csignal>
#include <iostream>
#include <unistd.h>
#include <cxxabi.h>
#include <string>
#include <sstream>
#include <string>
#include <dlfcn.h>
#include "../logging/logging.h"
#include <cpp-utils/process/SignalHandler.h>

// TODO Add file and line number on non-windows

using std::string;
using std::ostringstream;
using namespace cpputils::logging;

namespace cpputils {

namespace {
    std::string demangle(const string &mangledName) {
        string result;
        int status = -10;
        char *demangledName = nullptr;
        try {
            demangledName = abi::__cxa_demangle(mangledName.c_str(), NULL, NULL, &status);
            if (status == 0) {
                result = demangledName;
            } else {
                result = "[demangling error " + std::to_string(status) + "]" + mangledName;
            }
            free(demangledName);
            return result;
        } catch (...) {
            free(demangledName);
            throw;
        }
    }

    void pretty_print(std::ostream& str, const void *addr) {
        Dl_info info;
        if (0 == dladdr(addr, &info)) {
            str << "[failed parsing line]";
        } else {
            if (nullptr == info.dli_fname) {
                str << "[no dli_fname]";
            } else {
                str << info.dli_fname;
            }
            str << ":" << std::hex << info.dli_fbase << " ";
            if (nullptr == info.dli_sname) {
                str << "[no symbol name]";
            } else if (info.dli_sname[0] == '_') {
                // is a mangled name
                str << demangle(info.dli_sname);
            } else {
                // is not a mangled name
                str << info.dli_sname;
            }
            str << " : " << std::hex << info.dli_saddr;
        }
    }

    string backtrace_to_string(void *array[], size_t size) {
        ostringstream result;
        for (size_t i = 0; i < size; ++i) {
            result << "#" << std::dec << i << " ";
            pretty_print(result, array[i]);
            result << "\n";
        }
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
        LOG(ERR, "SIGSEGV\n{}", backtrace());
        exit(1);
    }
    void sigill_handler(int) {
        LOG(ERR, "SIGILL\n{}", backtrace());
        exit(1);
    }
    void sigabrt_handler(int) {
        LOG(ERR, "SIGABRT\n{}", backtrace());
        exit(1);
    }
}

	void showBacktraceOnCrash() {
        // the signal handler RAII objects will be initialized on first call (which will register the signal handler)
        // and destroyed on program exit (which will unregister the signal handler)

        static SignalHandlerRAII<&sigsegv_handler> segv(SIGSEGV);
        static SignalHandlerRAII<&sigabrt_handler> abrt(SIGABRT);
        static SignalHandlerRAII<&sigill_handler> ill(SIGILL);
	}
}

#endif
