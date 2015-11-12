#include "ThreadSystem.h"
#include "../logging/logging.h"

using std::function;
using namespace cpputils::logging;

namespace cpputils {

    ThreadSystem &ThreadSystem::singleton() {
        static ThreadSystem system;
        return system;
    }

    ThreadSystem::ThreadSystem(): _runningThreads(), _mutex() {
        //Stopping the thread before fork() (and then also restarting it in the parent thread after fork()) is important,
        //because as a running thread it might hold locks or condition variables that won't play well when forked.
        pthread_atfork(&ThreadSystem::_onBeforeFork, &ThreadSystem::_onAfterFork, &ThreadSystem::_onAfterFork);
    }

    ThreadSystem::Handle ThreadSystem::start(function<bool()> loopIteration) {
        boost::unique_lock<boost::mutex> lock(_mutex);
        auto thread = _startThread(loopIteration);
        _runningThreads.push_back(RunningThread{loopIteration, std::move(thread)});
        return std::prev(_runningThreads.end());
    }

    void ThreadSystem::stop(Handle handle) {
        boost::unique_lock<boost::mutex> lock(_mutex);
        boost::thread thread = std::move(handle->thread);
        thread.interrupt();
        _runningThreads.erase(handle);

        //It's fine if another thread gets the mutex while we still wait for the join. Joining doesn't change any internal state.
        lock.unlock();
        thread.join();
    }

    void ThreadSystem::_onBeforeFork() {
        singleton()._stopAllThreadsForRestart();
    }

    void ThreadSystem::_onAfterFork() {
        singleton()._restartAllThreads();
    }

    void ThreadSystem::_stopAllThreadsForRestart() {
        _mutex.lock(); // Is unlocked in the after-fork handler. This way, the whole fork() is protected.
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
        _mutex.unlock(); // Was locked in the before-fork handler
    }

    boost::thread ThreadSystem::_startThread(function<bool()> loopIteration) {
        return boost::thread(std::bind(&ThreadSystem::_runThread, loopIteration));
    }

    void ThreadSystem::_runThread(function<bool()> loopIteration) {
        try {
            bool cont = true;
            while(cont) {
                boost::this_thread::interruption_point();
                cont = loopIteration(); // This might also be interrupted.
            }
            //The thread is terminated gracefully.
        } catch (const boost::thread_interrupted &e) {
            //Do nothing, exit thread.
        } catch (const std::exception &e) {
            LOG(ERROR) << "LoopThread crashed: " << e.what();
        } catch (...) {
            LOG(ERROR) << "LoopThread crashed";
        }
        //TODO We should remove the thread from _runningThreads here, not in stop().
    }
}
