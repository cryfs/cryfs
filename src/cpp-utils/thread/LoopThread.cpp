#include "LoopThread.h"
#include "cpp-utils/thread/ThreadSystem.h"
#include <boost/none.hpp>
#include <stdexcept>
#include <string>
#include <utility>

using std::function;
using boost::none;

namespace cpputils {

    LoopThread::LoopThread(function<bool()> loopIteration, std::string threadName)
    : _loopIteration(std::move(loopIteration)), _runningHandle(none), _threadName(std::move(threadName)) {
    }

    LoopThread::~LoopThread() {
        if (_runningHandle != none) {
            stop();
        }
    }

    void LoopThread::start() {
        _runningHandle = ThreadSystem::singleton().start(_loopIteration, _threadName);
    }

    void LoopThread::stop() {
        if (_runningHandle == none) {
            throw std::runtime_error("LoopThread is not running");
        }
        ThreadSystem::singleton().stop(*_runningHandle);
        _runningHandle = none;
    }
}
