#include <cpp-utils/assert/backtrace.h>
#include <csignal>
#include <stdexcept>

#include <boost/config.hpp>

#if defined(_MSC_VER)
#include <Windows.h>
#else
#include <sys/resource.h>
#endif

namespace {
void install_crash_handler(const std::string &kind) {
	if (kind == "crash_signal") {
		cpputils::showBacktraceOnCrashSignals();
#if !defined(_MSC_VER)
		// In this mode we really do die from the signal and our caller checks that we do. Don't
		// leave a core dump behind for that, it would look like a real crash to whoever looks at
		// the directory the tests ran in.
		struct rlimit no_core_dumps{};
		no_core_dumps.rlim_cur = 0;
		no_core_dumps.rlim_max = 0;
		if (0 != ::setrlimit(RLIMIT_CORE, &no_core_dumps)) {
			throw std::runtime_error("Failed to disable core dumps");
		}
#endif
	} else {
		cpputils::showBacktraceOnCrash();
	}
}

}

// Deliberately neither static nor in an anonymous namespace, and deliberately not inlined: the
// backtrace tests check that the function we crash in shows up in the backtrace by name, and
// outside of Windows a symbolizer can only put a name to it if it has external linkage and ends up
// in the executable's dynamic symbol table (which is what ENABLE_EXPORTS in our cmake setup is for).
// NOLINTNEXTLINE(misc-use-internal-linkage)
BOOST_NOINLINE void handle_exit_signal(char **argv) {
	const std::string kind = argv[1];
	if (kind == "exception") {
		throw std::logic_error(argv[2]);
	} else if (kind == "nullptr") {
		int* ptr = nullptr;
		*ptr = 5; // NOLINT
	} else if (kind == "signal" || kind == "crash_signal") {
#if defined(_MSC_VER)
		DWORD code = std::atoll(argv[2]);
		::RaiseException(code, EXCEPTION_NONCONTINUABLE, 0, NULL);
#else
		const int code = static_cast<int>(std::strtol(argv[2], nullptr, 10));
		const int success = ::raise(code);
		if (success != 0) {
			throw std::runtime_error("Failed to raise signal");
		}
#endif
	}
}


int main(int  /*argc*/, char* argv[]) {
	install_crash_handler(argv[1]);
#if defined(_MSC_VER)
    // don't show windows error box
	_set_abort_behavior(0, _WRITE_ABORT_MSG);
#endif
	handle_exit_signal(argv);
	return 0;
}
