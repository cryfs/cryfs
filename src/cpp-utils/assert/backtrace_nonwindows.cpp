#if !defined(_MSC_VER)

#include <cerrno>
#include <csignal>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <stdexcept>
#include <string>

#include <pthread.h>
#include <unistd.h>

#include "backtrace.h"
#include "../logging/logging.h"
#include <cpp-utils/process/SignalHandler.h>

#include <boost/stacktrace.hpp>

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
// Set by showBacktraceOnCrashSignals(), see showBacktraceOnCrash().
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
bool crash_signal_handlers_installed = false;

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
    if (crash_signal_handlers_installed) {
        // showBacktraceOnCrashSignals() was installed first and we must not displace it: its
        // handlers don't call exit(), so the process still dies from the signal and still dumps
        // core, and it leaves SIGABRT alone, which ASSERT() and the gtest death tests need.
        // Test binaries install it in main() and then run production code that calls us, e.g.
        // cryfs-cli-test runs cryfs_cli::Cli::main() in-process; without this, they would silently
        // lose that behaviour from the first such test onwards.
        return;
    }

    // the signal handler RAII objects will be initialized on first call (which will register the signal handler)
    // and destroyed on program exit (which will unregister the signal handler)

    static const SignalHandlerRAII<&sigsegv_handler> segv(SIGSEGV);
    static const SignalHandlerRAII<&sigabrt_handler> abrt(SIGABRT);
    static const SignalHandlerRAII<&sigill_handler> ill(SIGILL);
}

namespace {
// How long we give backtrace() before we stop waiting for it. It is not async-signal-safe (see
// crash_signal_handler() below), so it can block forever, and a wedged process is worse than a
// missing backtrace.
constexpr unsigned int BACKTRACE_TIMEOUT_SECONDS = 10;

// The crash signal whose handler is currently running, so that the watchdog knows what to die
// from. volatile sig_atomic_t because that's the only kind of global a signal handler may touch.
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
volatile std::sig_atomic_t currently_handled_crash_signal = 0;

const char *crash_signal_name(int signal) {
    switch (signal) {
        case SIGSEGV: return "SIGSEGV";
        case SIGBUS: return "SIGBUS";
        case SIGILL: return "SIGILL";
        case SIGFPE: return "SIGFPE";
        default: return "UNKNOWN CRASH SIGNAL";
    }
}

// Everything from here to crash_signal_handler() runs inside a signal handler, so it may only use
// async-signal-safe functions. write(), sigaction(), sigemptyset(), sigaddset(), pthread_sigmask(),
// alarm(), raise() and _Exit() all are, see signal-safety(7).

// Write directly to the stderr file descriptor instead of going through the logger. write() is
// async-signal-safe and doesn't need any of the process state that the crash might have corrupted.
void write_to_stderr(const char *str, size_t size) {
    while (size > 0) {
        const ssize_t written = ::write(STDERR_FILENO, str, size);
        if (written <= 0) {
            if (written < 0 && EINTR == errno) {
                continue;
            }
            return;  // there's nothing sensible we could do about a failing write here
        }
        str += written;
        size -= static_cast<size_t>(written);
    }
}

void write_to_stderr(const char *str) {
    write_to_stderr(str, std::strlen(str));
}

bool reset_to_default_handler(int signal) {
    struct sigaction default_handler{};
    std::memset(&default_handler, 0, sizeof(default_handler));
    default_handler.sa_handler = SIG_DFL;  // NOLINT(cppcoreguidelines-pro-type-union-access)
    sigemptyset(&default_handler.sa_mask);
    return 0 == ::sigaction(signal, &default_handler, nullptr);
}

void unblock_signal(int signal) {
    sigset_t to_unblock{};
    sigemptyset(&to_unblock);
    sigaddset(&to_unblock, signal);
    // pthread_sigmask() and not sigprocmask(), because we can be running on any thread.
    static_cast<void>(::pthread_sigmask(SIG_UNBLOCK, &to_unblock, nullptr));
}

// Die from the crash signal we're currently handling. Its handler is back to SIG_DFL already (see
// crash_signal_handler()), but the signal is blocked while its own handler runs, so raising it
// would only mark it pending - unblock it first, then it kills us right here, with a core dump.
void die_from_crash_signal() {
    const int signal = currently_handled_crash_signal;
    unblock_signal(signal);
    static_cast<void>(::raise(signal));
    std::_Exit(EXIT_FAILURE);  // only reached if raise() didn't kill us after all
}

// Runs if backtrace() doesn't finish in time, e.g. because we crashed inside of the allocator while
// it held its lock and backtrace() now waits for that same lock.
void backtrace_timeout_handler(int) {
    write_to_stderr("Gave up on the backtrace, it didn't finish in time.\n");
    die_from_crash_signal();
}

void arm_backtrace_watchdog() {
    struct sigaction watchdog{};
    std::memset(&watchdog, 0, sizeof(watchdog));
    watchdog.sa_handler = &backtrace_timeout_handler;  // NOLINT(cppcoreguidelines-pro-type-union-access)
    sigemptyset(&watchdog.sa_mask);
    if (0 != ::sigaction(SIGALRM, &watchdog, nullptr)) {
        return;  // then we go without a watchdog, the backtrace is still worth trying
    }
    unblock_signal(SIGALRM);
    // We're about to die, so it doesn't matter that this throws away an alarm the program may have
    // set for itself.
    static_cast<void>(::alarm(BACKTRACE_TIMEOUT_SECONDS));
}

void crash_signal_handler(int signal) {
    const int saved_errno = errno;  // a signal handler must not clobber errno
    currently_handled_crash_signal = signal;

    // Reset to the default handler before doing anything else. That way, if we crash again while
    // printing the backtrace below, we still die from the signal instead of misbehaving in here,
    // and the re-raise at the end of this function can't call us again in a loop.
    if (!reset_to_default_handler(signal)) {
        // can only fail for an invalid signal number, i.e. never
        write_to_stderr("Failed to restore the default signal handler\n");
        std::_Exit(EXIT_FAILURE);
    }

    write_to_stderr(crash_signal_name(signal));
    write_to_stderr("\n");

    // backtrace() symbolizes into a std::string, i.e. it allocates, and is therefore not
    // async-signal-safe: if we took the signal inside of the allocator - a segfault in malloc/free
    // is exactly what heap corruption looks like - it deadlocks on the allocator lock. Symbolized
    // frames are the whole point of this handler, so we do call it, but under a watchdog, so that
    // such a crash still dies from the signal (and dumps core) a few seconds later instead of
    // hanging a CI job until its timeout. The signal name above is already out, it went through
    // plain write(2) and doesn't depend on any of this.
    arm_backtrace_watchdog();
    {
        const std::string trace = backtrace();
        write_to_stderr(trace.c_str(), trace.size());
    }  // the destructor frees, which can block just like the allocation did, so disarm only here
    static_cast<void>(::alarm(0));

    // Re-raise the signal so we terminate with the correct wait status (i.e. killed by `signal`)
    // and still write a core dump. `signal` is blocked while its own handler runs, so the re-raised
    // signal is delivered once we return from here - by then the default handler is in place, so it
    // kills us instead of calling us again. (For a real fault, as opposed to a raise(), returning
    // re-executes the faulting instruction and ends up in the default handler as well.)
    errno = saved_errno;
    static_cast<void>(::raise(signal));  // raise() only fails for an invalid signal number
}

void install_crash_signal_handler(int signal) {
    struct sigaction new_handler{};
    std::memset(&new_handler, 0, sizeof(new_handler));
    new_handler.sa_handler = &crash_signal_handler;  // NOLINT(cppcoreguidelines-pro-type-union-access)
    new_handler.sa_flags = SA_RESTART;
    // Deliberately an empty mask and not sigfillset() the way SignalHandlerRAII does it. Our
    // handler calls backtrace(), which can block indefinitely (see above). Blocking every signal
    // for that would mean nothing short of SIGKILL could reclaim such a process - not SIGTERM, not
    // the `timeout` a CI job runs its tests under - and our own SIGALRM watchdog couldn't run
    // either. `signal` itself is blocked while its handler runs anyway, and a different crash
    // signal arriving meanwhile finds the default handler and kills us, which is what we want.
    sigemptyset(&new_handler.sa_mask);
    if (0 != ::sigaction(signal, &new_handler, nullptr)) {
        throw std::runtime_error("Error calling sigaction. Errno: " + std::to_string(errno));
    }
}
}

void showBacktraceOnCrashSignals() {
    // Note: deliberately no handler for SIGABRT, see the comment in backtrace.h.
    // Unlike showBacktraceOnCrash(), these handlers stay installed until the process dies instead
    // of being removed at exit - a crash during static destruction should print a backtrace too.
    install_crash_signal_handler(SIGSEGV);
    install_crash_signal_handler(SIGBUS);
    install_crash_signal_handler(SIGILL);
    install_crash_signal_handler(SIGFPE);
    crash_signal_handlers_installed = true;
}

}

#endif
