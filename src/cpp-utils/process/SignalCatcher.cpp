#include "SignalCatcher.h"
#include "SignalHandler.h"

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

class SignalCatcherRegistry final {
public:
    void add(int signal, details::SignalCatcherImpl* signal_occurred_flag) {
        _catchers.write([&] (auto& catchers) {
            catchers.emplace_back(signal, signal_occurred_flag);
        });
    }

    void remove(details::SignalCatcherImpl* catcher) {
        _catchers.write([&] (auto& catchers) {
            auto found = std::find_if(catchers.rbegin(), catchers.rend(), [catcher] (const auto& entry) {return entry.second == catcher;});
            ASSERT(found != catchers.rend(), "Signal handler not found");
            catchers.erase(--found.base()); // decrement because it's a reverse iterator
        });
    }

    ~SignalCatcherRegistry() {
        ASSERT(0 == _catchers.read([] (auto& catchers) {return catchers.size();}), "Leftover signal catchers that weren't destroyed");
    }

	details::SignalCatcherImpl* find(int signal) {
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
    LeftRight<vector<pair<int, details::SignalCatcherImpl*>>> _catchers;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherRegistry);
};

class SignalCatcherRegisterer final {
public:
    SignalCatcherRegisterer(int signal, details::SignalCatcherImpl* catcher)
    : _catcher(catcher) {
        SignalCatcherRegistry::singleton().add(signal, _catcher);
    }

    ~SignalCatcherRegisterer() {
        SignalCatcherRegistry::singleton().remove(_catcher);
    }

private:
    details::SignalCatcherImpl* _catcher;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherRegisterer);
};

}

namespace details {

class SignalCatcherImpl final {
public:
    SignalCatcherImpl(int signal, std::atomic<bool>* signal_occurred_flag)
    : _signal_occurred_flag(signal_occurred_flag)
    , _registerer(signal, this)
    , _handler(signal) {
        // note: the order of the members ensures that:
        //  - when registering the signal handler, the SignalCatcher impl already has a valid _signal_occurred_flag set.
        //  - when registering the signal handler fails, the _registerer will be destroyed again, unregistering this SignalCatcherImpl,
        //    i.e. there is no leak.

        // Allow only the set of signals that is supported on all platforms, see for Windows: https://docs.microsoft.com/en-us/cpp/c-runtime-library/reference/signal?view=vs-2017
        ASSERT(signal == SIGABRT || signal == SIGFPE || signal == SIGILL || signal == SIGINT || signal == SIGSEGV || signal == SIGTERM, "Unknown signal");
    }

    void setSignalOccurred() {
        *_signal_occurred_flag = true;
    }
private:
    std::atomic<bool>* _signal_occurred_flag;
    SignalCatcherRegisterer _registerer;
    SignalHandlerRAII<&got_signal> _handler;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcherImpl);
};

}

namespace {
void got_signal(int signal) {
    SignalCatcherRegistry::singleton().find(signal)->setSignalOccurred();
}
}

SignalCatcher::SignalCatcher(std::initializer_list<int> signals)
: _signal_occurred(false)
, _impls() {
    // note: the order of the members ensures that:
    //  - when the signal handler is set, the _signal_occurred flag is already initialized.
    //  - the _signal_occurred flag will not be destructed as long as the signal handler might be called (i.e. as long as _impls lives)

    _impls.reserve(signals.size());
    for (const int signal : signals) {
        _impls.emplace_back(make_unique<details::SignalCatcherImpl>(signal, &_signal_occurred));
    }
}

SignalCatcher::~SignalCatcher() {}

}
