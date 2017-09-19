#include "LoopThread.h"
#include "../logging/logging.h"

using std::function;
using boost::none;

namespace cpputils {

    LoopThread::LoopThread(function<bool()> loopIteration)
        : _loopIteration(loopIteration), _runningHandle(none) {
    }

    LoopThread::~LoopThread() {
        if (_runningHandle != none) {
            stop();
        }
    }

    LoopThread::LoopThread(LoopThread&& rhs)
        : _loopIteration(std::move(rhs._loopIteration)), _runningHandle(std::move(rhs._runningHandle)) {
        rhs._runningHandle = none; // don't destroy the thread in the original destructor
    }

    void LoopThread::start() {
        _runningHandle = ThreadSystem::singleton().start(_loopIteration);
    }

    void LoopThread::stop() {
        if (_runningHandle == none) {
            throw std::runtime_error("LoopThread is not running");
        }
        ThreadSystem::singleton().stop(*_runningHandle);
        _runningHandle = none;
    }
}
