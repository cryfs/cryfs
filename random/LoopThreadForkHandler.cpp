#include "LoopThreadForkHandler.h"
#include <thread>
#include "../logging/logging.h"
#include "../assert/assert.h"
#include "LoopThread.h"

using namespace cpputils::logging;

namespace cpputils {
    LoopThreadForkHandler &LoopThreadForkHandler::singleton() {
        static LoopThreadForkHandler singleton;
        return singleton;
    }

    LoopThreadForkHandler::LoopThreadForkHandler() {
        //pthread_atfork(&LoopThreadForkHandler::_onBeforeFork, &LoopThreadForkHandler::_onAfterFork, &LoopThreadForkHandler::_onAfterFork);
        pthread_atfork(nullptr, nullptr, &LoopThreadForkHandler::_onAfterFork);
    }

    void LoopThreadForkHandler::add(LoopThread *thread) {
        _runningThreads.push_back(thread);
    }

    void LoopThreadForkHandler::remove(LoopThread *thread) {
        auto found = std::find(_runningThreads.begin(), _runningThreads.end(), thread);
        ASSERT(found != _runningThreads.end(), "Thread not found");
        _runningThreads.erase(found);
    }

    void LoopThreadForkHandler::_onBeforeFork() {
        singleton()._stopThreads();
    }

    void LoopThreadForkHandler::_stopThreads() {
        for (LoopThread *thread : _runningThreads) {
            thread->asyncStop();
        }
        for (LoopThread *thread : _runningThreads) {
            thread->waitUntilStopped();
        }
    }

    void LoopThreadForkHandler::_onAfterFork() {
        singleton()._startThreads();
    }

    void LoopThreadForkHandler::_startThreads() {
        for (LoopThread *thread : _runningThreads) {
            thread->start();
        }
    }

}
