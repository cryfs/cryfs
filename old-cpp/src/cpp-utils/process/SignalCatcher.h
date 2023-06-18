#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SIGNALCATCHER_H_
#define MESSMER_CPPUTILS_PROCESS_SIGNALCATCHER_H_

#include <cpp-utils/macros.h>
#include <atomic>
#include <csignal>
#include <memory>
#include <vector>

namespace cpputils {

namespace details {
class SignalCatcherImpl;
}

/*
 * While an instance of this class is in scope, the specified signal (e.g. SIGINT)
 * is caught and doesn't exit the application. You can poll if the signal occurred.
 */
class SignalCatcher final {
public:
    SignalCatcher(): SignalCatcher({SIGINT, SIGTERM}) {}

    SignalCatcher(std::initializer_list<int> signals);
    ~SignalCatcher();

    bool signal_occurred() const {
        return _signal_occurred;
    }

private:
    // note: _signal_occurred must be initialized before _impl because _impl might use it
    std::atomic<bool> _signal_occurred;
    std::vector<std::unique_ptr<details::SignalCatcherImpl>> _impls;

    DISALLOW_COPY_AND_ASSIGN(SignalCatcher);
};


}

#endif
