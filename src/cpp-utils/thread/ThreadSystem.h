#pragma once
#ifndef MESSMER_CPPUTILS_THREAD_THREADSYSTEM_H
#define MESSMER_CPPUTILS_THREAD_THREADSYSTEM_H

#include "../macros.h"
#include <boost/thread.hpp>
#include <list>
#include <functional>

namespace cpputils {
    //TODO Test

    class ThreadSystem final {
    private:
        struct RunningThread {
            RunningThread(RunningThread&&) = default;
            RunningThread(const RunningThread&) = delete;
            RunningThread& operator=(const RunningThread&) = delete;

            std::string threadName;
            std::function<bool()> loopIteration;  // The loopIteration callback returns true, if more iterations should be run, and false, if the thread should be terminated.
            boost::thread thread;  // boost::thread because we need it to be interruptible.
        };
    public:
        using Handle = std::list<RunningThread>::iterator;

        static ThreadSystem &singleton();

        Handle start(std::function<bool()> loopIteration, std::string threadName);
        void stop(Handle handle);

    private:
        ThreadSystem();

        static void _runThread(std::function<bool()> loopIteration);

        static void _onBeforeFork();
        static void _onAfterFork();
        //TODO Rename to _doOnBeforeFork and _doAfterFork or similar, because they also handle locking _mutex for fork().
        void _stopAllThreadsForRestart();
        void _restartAllThreads();
        boost::thread _startThread(std::function<bool()> loopIteration, const std::string& threadName);

        std::list<RunningThread> _runningThreads;  // std::list, because we give out iterators as handles
        boost::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(ThreadSystem);
    };
}

#endif
