#include <boost/stacktrace/stacktrace.hpp>
#include <cstdlib>
#include <string>
#if !defined(_MSC_VER)

#include <csignal>
#include <sstream>

#include "../logging/logging.h"
#include <cpp-utils/process/SignalHandler.h>


using std::string;
using std::ostringstream;
using namespace cpputils::logging;

namespace cpputils {

string backtrace() {
    std::ostringstream str;
    str << boost::stacktrace::stacktrace();
    return str.str();
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

    static const SignalHandlerRAII<&sigsegv_handler> segv(SIGSEGV);
    static const SignalHandlerRAII<&sigabrt_handler> abrt(SIGABRT);
    static const SignalHandlerRAII<&sigill_handler> ill(SIGILL);
}

}

#endif
