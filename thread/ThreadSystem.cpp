#include "ThreadSystem.h"
#include "../logging/logging.h"

using std::function;
using namespace cpputils::logging;

namespace cpputils {

    ThreadSystem &ThreadSystem::singleton() {
        static ThreadSystem system;
        return system;
    }

    ThreadSystem::ThreadSystem(): _runningThreads() {
        //Stopping the thread before fork() (and then also restarting it in the parent thread after fork()) is important,
        //because as a running thread it might hold locks or condition variables that won't play well when forked.
        pthread_atfork(&ThreadSystem::_onBeforeFork, &ThreadSystem::_onAfterFork, &ThreadSystem::_onAfterFork);
    }

    ThreadSystem::Handle ThreadSystem::start(function<void()> loopIteration) {
        auto thread = _startThread(loopIteration);
        _runningThreads.push_back(RunningThread{loopIteration, std::move(thread)});
        return std::prev(_runningThreads.end());
    }

    void ThreadSystem::stop(Handle handle) {
        handle->thread.interrupt();
        handle->thread.join();
        _runningThreads.erase(handle);
    }

    void ThreadSystem::_onBeforeFork() {
        singleton()._stopAllThreadsForRestart();
    }

    void ThreadSystem::_onAfterFork() {
        singleton()._restartAllThreads();
    }

    void ThreadSystem::_stopAllThreadsForRestart() {
        for (RunningThread &thread : _runningThreads) {
            thread.thread.interrupt();
        }
        for (RunningThread &thread : _runningThreads) {
            thread.thread.join();
        }
    }

    void ThreadSystem::_restartAllThreads() {
        for (RunningThread &thread : _runningThreads) {
            thread.thread = _startThread(thread.loopIteration);
        }
    }

    boost::thread ThreadSystem::_startThread(function<void()> loopIteration) {
        return boost::thread(std::bind(&ThreadSystem::_runThread, loopIteration));
    }

    void ThreadSystem::_runThread(function<void()> loopIteration) {
        try {
            while(true) {
                boost::this_thread::interruption_point();
                loopIteration(); // This might also be interrupted.
            }
        } catch (const boost::thread_interrupted &e) {
            //Do nothing, exit thread.
        } catch (const std::exception &e) {
            LOG(ERROR) << "LoopThread crashed: " << e.what();
        } catch (...) {
            LOG(ERROR) << "LoopThread crashed";
        }
    }
}
