#pragma once
#ifndef MESSMER_CRYFS_SRC_CLI_CALLAFTERTIMEOUT_H
#define MESSMER_CRYFS_SRC_CLI_CALLAFTERTIMEOUT_H

#include <functional>
#include <mutex>
#include <cpp-utils/thread/LoopThread.h>

namespace cryfs {
    class CallAfterTimeout final {
    public:
        CallAfterTimeout(boost::chrono::milliseconds timeout, std::function<void()> callback);
        void resetTimer();
    private:
        bool _checkTimeoutThreadIteration();
        boost::chrono::time_point<boost::chrono::steady_clock> _targetTime();
        bool _callCallbackIfTimeout();

        std::function<void()> _callback;
        boost::chrono::milliseconds _timeout;
        boost::chrono::time_point<boost::chrono::steady_clock> _start;
        cpputils::LoopThread _checkTimeoutThread;
        std::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(CallAfterTimeout);
    };

    inline CallAfterTimeout::CallAfterTimeout(boost::chrono::milliseconds timeout, std::function<void()> callback)
        :_callback(callback), _timeout(timeout), _start(), _checkTimeoutThread(std::bind(&CallAfterTimeout::_checkTimeoutThreadIteration, this)) {
        resetTimer();
        _checkTimeoutThread.start();
    }

    inline void CallAfterTimeout::resetTimer() {
        std::unique_lock<std::mutex> lock(_mutex);
        _start = boost::chrono::steady_clock::now();
    }

    inline bool CallAfterTimeout::_checkTimeoutThreadIteration() {
        boost::this_thread::sleep_until(_targetTime());
        return _callCallbackIfTimeout();
    }

    inline boost::chrono::time_point<boost::chrono::steady_clock> CallAfterTimeout::_targetTime() {
        std::unique_lock<std::mutex> lock(_mutex);
        return _start + _timeout;
    }

    inline bool CallAfterTimeout::_callCallbackIfTimeout() {
        std::unique_lock<std::mutex> lock(_mutex);
        if (boost::chrono::steady_clock::now() >= _start + _timeout) {
            _callback();
            return false; // Stop thread
        }
        return true; // Continue thread
    }
}

#endif
