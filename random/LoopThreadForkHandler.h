#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_LOOPTHREADFORKHANDLER_H
#define MESSMER_CPPUTILS_RANDOM_LOOPTHREADFORKHANDLER_H

#include <vector>
#include "../macros.h"

namespace cpputils {
    class LoopThread;

    // The fork() syscall only forks the main thread.
    // This class takes care that LoopThreads are also run in the child process.
    class LoopThreadForkHandler {
    public:
        static LoopThreadForkHandler &singleton();

        void add(LoopThread *thread);
        void remove(LoopThread *thread);

    private:
        LoopThreadForkHandler();
        static void _onBeforeFork();
        static void _onAfterFork();

        void _startThreads();
        void _stopThreads();

        std::vector<LoopThread*> _runningThreads;

        DISALLOW_COPY_AND_ASSIGN(LoopThreadForkHandler);
    };
}

#endif
