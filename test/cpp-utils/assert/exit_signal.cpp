#include <cpp-utils/assert/backtrace.h>
#include <csignal>
#include <stdexcept>

#if defined(_MSC_VER)
#include <Windows.h>
#endif

void handle_exit_signal(char **argv) {
	const std::string kind = argv[1];
	if (kind == "exception") {
		throw std::logic_error(argv[2]);
	} else if (kind == "nullptr") {
		int* ptr = nullptr;
		*ptr = 5; // NOLINT
	} else if (kind == "signal") {
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
	cpputils::showBacktraceOnCrash();
#if defined(_MSC_VER)
    // don't show windows error box
	_set_abort_behavior(0, _WRITE_ABORT_MSG);
#endif
	handle_exit_signal(argv);
	return 0;
}
