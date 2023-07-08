#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SIGNALHANDLER_H_
#define MESSMER_CPPUTILS_PROCESS_SIGNALHANDLER_H_

#include <memory>
#include <csignal>
#include <cpp-utils/assert/assert.h>

// TODO Test SignalHandler

/**
 * A SignalHandlerRAII instance replaces the signal handler for the given signal with the given handler
 * as long as it is alive and sets it to the previous handler once it dies.
 * This way, it can be used for stacking different signal handlers on top of each other.
 */

namespace cpputils {

using SignalHandlerFunction = void(int);

#if !defined(_MSC_VER)

template<SignalHandlerFunction* handler>
class SignalHandlerRAII final {
public:
    explicit SignalHandlerRAII(int signal)
            : _old_handler(), _signal(signal) {
        struct sigaction new_signal_handler{};
        std::memset(&new_signal_handler, 0, sizeof(new_signal_handler));
        new_signal_handler.sa_handler = handler;  // NOLINT(cppcoreguidelines-pro-type-union-access)
        new_signal_handler.sa_flags = SA_RESTART;
        const int error = sigfillset(&new_signal_handler.sa_mask);  // block all signals while signal handler is running
        if (0 != error) {
            throw std::runtime_error("Error calling sigfillset. Errno: " + std::to_string(errno));
        }
        _sigaction(_signal, &new_signal_handler, &_old_handler);
    }

    ~SignalHandlerRAII() {
        // reset to old signal handler
        struct sigaction removed_handler{};
        _sigaction(_signal, &_old_handler, &removed_handler);
        if (handler != removed_handler.sa_handler) {  // NOLINT(cppcoreguidelines-pro-type-union-access)
            ASSERT(false, "Signal handler screwup. We just replaced a signal handler that wasn't our own.");
        }
    }

private:
    static void _sigaction(int signal, struct sigaction *new_handler, struct sigaction *old_handler) {
        const int error = sigaction(signal, new_handler, old_handler);
        if (0 != error) {
            throw std::runtime_error("Error calling sigaction. Errno: " + std::to_string(errno));
        }
    }

    struct sigaction _old_handler;
    int _signal;

    DISALLOW_COPY_AND_ASSIGN(SignalHandlerRAII);
};

#else
namespace details {
// The Linux default behavior (i.e. the way we set up sigaction above) is to disable signal processing while the signal
// handler is running and to re-enable the custom handler once processing is finished. The Windows default behavior
// is to reset the handler to the default handler directly before executing the handler, i.e. the handler will only
// be called once. To fix this, we use this RAII class on Windows, of which an instance will live in the signal handler.
// In its constructor, it disables signal handling, and in its destructor it re-sets the custom handler.
// This is not perfect since there is a small time window between calling the signal handler and calling the constructor
// of this class, but it's the best we can do.
template<SignalHandlerFunction* handler>
class SignalHandlerRunningRAII final {
public:
    explicit SignalHandlerRunningRAII(int signal) : _signal(signal) {
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
        SignalHandlerFunction* old_handler = ::signal(_signal, &details::wrap_signal_handler<handler>);
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

template<SignalHandlerFunction* handler>
void wrap_signal_handler(int signal) {
    SignalHandlerRunningRAII<handler> disable_signal_processing_while_handler_running_and_reset_handler_afterwards(signal);
    (*handler)(signal);
}
}

template<SignalHandlerFunction* handler>
class SignalHandlerRAII final {
public:
    explicit SignalHandlerRAII(int signal)
    : _old_handler(nullptr), _signal(signal) {
        _old_handler = ::signal(_signal, &details::wrap_signal_handler<handler>);
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
        if (error != &details::wrap_signal_handler<handler>) {
            throw std::logic_error("Signal handler screwup. We just replaced a signal handler that wasn't our own.");
        }
    }

private:

    SignalHandlerFunction* _old_handler;
    int _signal;

    DISALLOW_COPY_AND_ASSIGN(SignalHandlerRAII);
};

#endif

}

#endif
