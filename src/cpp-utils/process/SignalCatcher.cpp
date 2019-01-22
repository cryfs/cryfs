#include "SignalCatcher.h"

#include <algorithm>
#include <stdexcept>
#include <vector>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/thread/LeftRight.h>

using std::make_unique;
using std::vector;
using std::pair;

namespace cpputils {

namespace {

void got_signal(int signal);

using SignalHandlerFunction = void(int);

constexpr SignalHandlerFunction* signal_catcher_function = &got_signal;

#if !defined(_MSC_VER)

class SignalHandlerRAII final {
public:
	SignalHandlerRAII(int signal)
		: _old_handler(), _signal(signal) {
		struct sigaction new_signal_handler{};
		std::memset(&new_signal_handler, 0, sizeof(new_signal_handler));
		new_signal_handler.sa_handler = signal_catcher_function;  // NOLINT(cppcoreguidelines-pro-type-union-access)
		new_signal_handler.sa_flags = SA_RESTART;
		int error = sigfillset(&new_signal_handler.sa_mask);  // block all signals while signal handler is running
		if (0 != error) {
			throw std::runtime_error("Error calling sigfillset. Errno: " + std::to_string(errno));
		}
		_sigaction(_signal, &new_signal_handler, &_old_handler);
	}

	~SignalHandlerRAII() {
		// reset to old signal handler
		struct sigaction removed_handler{};
		_sigaction(_signal, &_old_handler, &removed_handler);
		if (signal_catcher_function != removed_handler.sa_handler) {  // NOLINT(cppcoreguidelines-pro-type-union-access)
			ASSERT(false, "Signal handler screwup. We just replaced a signal handler that wasn't our own.");
		}
	}

private:
	static void _sigaction(int signal, struct sigaction *new_handler, struct sigaction *old_handler) {
		int error = sigaction(signal, new_handler, old_handler);
		if (0 != error) {
			throw std::runtime_error("Error calling sigaction. Errno: " + std::to_string(errno));
		}
	}

	struct sigaction _old_handler;
	int _signal;

	DISALLOW_COPY_AND_ASSIGN(SignalHandlerRAII);
};

#else

class SignalHandlerRAII final {
public:
	SignalHandlerRAII(int signal)
		: _old_handler(nullptr), _signal(signal) {
		_old_handler = ::signal(_signal, signal_catcher_function);
		if (_old_handler == SIG_ERR) {
			throw std::logic_error("Error calling signal(). Errno: " + std::to_string(errno));
		}
	}

	~SignalHandlerRAII() {
		// reset to old signal handler
		SignalHandlerFunction* error = ::signal(_signal, _old_handler);
		if (error == SIG_ERR) {
			throw std::logic_error("Error resetting signal(). Errno: " + std::to_string(errno));
		}
		if (error != signal_catcher_function) {
			throw std::logic_error("Signal handler screwup. We just replaced a signal handler that wasn't our own.");
		}
	}

private:

	SignalHandlerFunction* _old_handler;
	int _signal;

	DISALLOW_COPY_AND_ASSIGN(SignalHandlerRAII);
};

// The Linux default behavior (i.e. the way we set up sigaction above) is to disable signal processing while the signal
// handler is running and to re-enable the custom handler once processing is finished. The Windows default behavior
// is to reset the handler to the default handler directly before executing the handler, i.e. the handler will only
// be called once. To fix this, we use this RAII class on Windows, of which an instance will live in the signal handler.
// In its constructor, it disables signal handling, and in its destructor it re-sets the custom handler.
// This is not perfect since there is a small time window between calling the signal handler and calling the constructor
// of this class, but it's the best we can do.
class SignalHandlerRunningRAII final {
public:
	SignalHandlerRunningRAII(int signal) : _signal(signal) {
		SignalHandlerFunction* old_handler = ::signal(_signal, SIG_IGN);
		if (old_handler == SIG_ERR) {
			throw std::logic_error("Error disabling signal(). Errno: " + std::to_string(errno));
		}
		if (old_handler != SIG_DFL) {
			// see description above, we expected the signal handler to be reset.
			throw std::logic_error("We expected windows to reset the signal handler but it didn't. Did the Windows API change?");
		}
	}

	~SignalHandlerRunningRAII() {
		SignalHandlerFunction* old_handler = ::signal(_signal, signal_catcher_function);
		if (old_handler == SIG_ERR) {
			throw std::logic_error("Error resetting signal() after calling handler. Errno: " + std::to_string(errno));
		}
		if (old_handler != SIG_IGN) {
			throw std::logic_error("Weird, we just did set the signal handler to ignore. Why isn't it still ignore?");
		}
	}

private:
	int _signal;
};

#endif

class SignalCatcherRegistry final {
public:
    void add(int signal, std::atomic<bool>* signal_occurred_flag) {
        _catchers.write([&] (auto& catchers) {
            catchers.emplace_back(signal, signal_occurred_flag);
        });
    }

    void remove(std::atomic<bool>* catcher) {
        _catchers.write([&] (auto& catchers) {
            auto found = std::find_if(catchers.rbegin(), catchers.rend(), [catcher] (const auto& entry) {return entry.second == catcher;});
            ASSERT(found != catchers.rend(), "Signal handler not found");
            catchers.erase(--found.base()); // decrement because it's a reverse iterator
        });
    }

    ~SignalCatcherRegistry() {
        ASSERT(0 == _catchers.read([] (auto& catchers) {return catchers.size();}), "Leftover signal catchers that weren't destroyed");
    }

	std::atomic<bool>* find(int signal) {
		// this is called in a signal handler and must be mutex-free.
		return _catchers.read([&](auto& catchers) {
			auto found = std::find_if(catchers.rbegin(), catchers.rend(), [signal](const auto& entry) {return entry.first == signal; });
			ASSERT(found != catchers.rend(), "Signal handler not found");
			return found->second;
		});
	}

    static SignalCatcherRegistry& singleton() {
        static SignalCatcherRegistry _singleton;
        return _singleton;
    }

private:
    SignalCatcherRegistry() = default;

    // using LeftRight datastructure because we need mutex-free reads. Signal handlers can't use mutexes.
    LeftRight<vector<pair<int, std::atomic<bool>*>>> _catchers;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherRegistry);
};

void got_signal(int signal) {
#if defined(_MSC_VER)
	// Only needed on Windows, Linux does this by default. See comment on SignalHandlerRunningRAII class.
	SignalHandlerRunningRAII disable_signal_processing_while_handler_running_and_reset_handler_afterwards(signal);
#endif
	std::atomic<bool>* catcher = SignalCatcherRegistry::singleton().find(signal);
	*catcher = true;
}

class SignalCatcherRegisterer final {
public:
    SignalCatcherRegisterer(int signal, std::atomic<bool>* catcher)
    : _catcher(catcher) {
        SignalCatcherRegistry::singleton().add(signal, _catcher);
    }

    ~SignalCatcherRegisterer() {
        SignalCatcherRegistry::singleton().remove(_catcher);
    }

private:
    std::atomic<bool>* _catcher;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherRegisterer);
};

}

namespace details {

class SignalCatcherImpl final {
public:
    SignalCatcherImpl(int signal, std::atomic<bool>* signal_occurred_flag)
    : _registerer(signal, signal_occurred_flag)
    , _handler(signal) {
        // note: the order of the members ensures that:
        //  - when registering the signal handler fails, the registerer will be destroyed, unregistering the signal_occurred_flag,
        //    i.e. there is no leak.

        // Allow only the set of signals that is supported on all platforms, see for Windows: https://docs.microsoft.com/en-us/cpp/c-runtime-library/reference/signal?view=vs-2017
        ASSERT(signal == SIGABRT || signal == SIGFPE || signal == SIGILL || signal == SIGINT || signal == SIGSEGV || signal == SIGTERM, "Unknown signal");
    }
private:
    SignalCatcherRegisterer _registerer;
    SignalHandlerRAII _handler;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherImpl);
};

}

SignalCatcher::SignalCatcher(std::initializer_list<int> signals)
: _signal_occurred(false)
, _impls() {
    // note: the order of the members ensures that:
    //  - when the signal handler is set, the _signal_occurred flag is already initialized.
    //  - the _signal_occurred flag will not be destructed as long as the signal handler might be called (i.e. as long as _impls lives)

    _impls.reserve(signals.size());
    for (int signal : signals) {
        _impls.emplace_back(make_unique<details::SignalCatcherImpl>(signal, &_signal_occurred));
    }
}

SignalCatcher::~SignalCatcher() {}

}
