#pragma once
#ifndef MESSMER_CPPUTILS_THREAD_LOOPTHREAD_H
#define MESSMER_CPPUTILS_THREAD_LOOPTHREAD_H

#include "ThreadSystem.h"
#include <boost/optional.hpp>

namespace cpputils {
    //TODO Test
    //TODO Test that fork() doesn't destroy anything (e.g. no deadlock on stop() because thread is not running anymore)

    // Has to be final, because otherwise there could be a race condition where LoopThreadForkHandler calls a LoopThread
    // where the child class destructor already ran.
    class LoopThread final {
    public:
        LoopThread(std::function<void()> loopIteration);
        ~LoopThread();
        void start();
        void stop();

    private:
        std::function<void()> _loopIteration;
        boost::optional<ThreadSystem::Handle> _runningHandle;
    };
}

#endif
