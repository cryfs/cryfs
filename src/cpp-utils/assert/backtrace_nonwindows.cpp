#if !defined(_MSC_VER)

#include <csignal>
#include <cxxabi.h>
#include <sstream>

#include "../logging/logging.h"
#include <cpp-utils/process/SignalHandler.h>

#define UNW_LOCAL_ONLY
#include <libunwind.h>

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
            } else if (status == -2) {
                // mangledName was not a c++ mangled name, probably because it's a C name like for static
                // initialization or stuff. Let's just return the name instead.
                result = mangledName;
            } else {
                // other error
                result = "[demangling error " + std::to_string(status) + "]" + mangledName;
            }
            free(demangledName);
            return result;
        } catch (...) {
            free(demangledName);
            throw;
        }
    }

    void pretty_print(std::ostringstream& str, unw_cursor_t* cursor) {
        constexpr unsigned int MAXNAMELEN=256;
        char name[MAXNAMELEN];
        unw_word_t offp = 0, ip = 0;

        int status = unw_get_reg(cursor, UNW_REG_IP, &ip);
        if (0 != status) {
            str << "[unw_get_reg error: " << status << "]: ";
        } else {
            str << "0x" << std::hex << ip << ": ";
        }

        status = unw_get_proc_name(cursor, name, MAXNAMELEN, &offp);
        if (0 != status) {
            str << "[unw_get_proc_name error: " << status << "]";
        } else {
            str << demangle(name);
        }
        str << " +0x" << std::hex << offp;
    }
}

	string backtrace() {
        std::ostringstream result;

        unw_context_t uc;
        int status = unw_getcontext(&uc);
        if (0 != status) {
            return "[unw_getcontext error: " + std::to_string(status) + "]";
        }

        unw_cursor_t cursor;
        status = unw_init_local(&cursor, &uc);
        if (0 != status) {
            return "[unw_init_local error: " + std::to_string(status) + "]";
        }


        size_t line = 0;
        while ((status = unw_step(&cursor)) > 0) {
            result << "#" << std::dec << (line++) << " ";
            pretty_print(result, &cursor);
            result << "\n";
        }
        if (status != 0) {
            result << "[unw_step error :" << status << "]";
        }

        return result.str();
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
