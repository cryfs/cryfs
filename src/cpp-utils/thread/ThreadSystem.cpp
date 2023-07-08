#include "ThreadSystem.h"
#include "../logging/logging.h"
#include "debugging.h"

using std::function;
using std::string;
using namespace cpputils::logging;

namespace cpputils {

    ThreadSystem &ThreadSystem::singleton() {
        static ThreadSystem system;
        return system;
    }

    ThreadSystem::ThreadSystem(): _runningThreads(), _mutex() {
#if !defined(_MSC_VER)
        //Stopping the thread before fork() (and then also restarting it in the parent thread after fork()) is important,
        //because as a running thread it might hold locks or condition variables that won't play well when forked.
        pthread_atfork(&ThreadSystem::_onBeforeFork, &ThreadSystem::_onAfterFork, &ThreadSystem::_onAfterFork);
#else
		// not needed on windows because we don't fork
#endif
    }

    ThreadSystem::Handle ThreadSystem::start(function<bool()> loopIteration, string threadName) {
        const boost::unique_lock<boost::mutex> lock(_mutex);
        auto thread = _startThread(loopIteration, threadName);
        _runningThreads.push_back(RunningThread{std::move(threadName), std::move(loopIteration), std::move(thread)});
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
            if (boost::this_thread::get_id() == thread.thread.get_id()) {
                // This means fork was called from within one of our _runningThreads.
                // We cannot wait or ourselves to die.
                // Forking from within a thread is usually chaos since the forked process only gets a copy
                // of the calling thread as its new main thread. So we (hopefully) never should do this.
                // This is, however, a valid pattern when fork() is directly followed by an exec().
                // So let's just ignore this situation and continue as if nothing happened, assuming an exec()
                // follows soon.
                continue;
            }
            thread.thread.interrupt();
        }
        for (RunningThread &thread : _runningThreads) {
            if (boost::this_thread::get_id() == thread.thread.get_id()) {
                // This means fork was called from within one of our _runningThreads. See comment above.
                continue;
            }
            thread.thread.join();
        }
    }

    void ThreadSystem::_restartAllThreads() {
        for (RunningThread &thread : _runningThreads) {
            if (thread.thread.joinable()) {
                // Because all non-self threads have been terminated in _stopAllThreadsForRestart,
                // this means fork was called from within one of our _runningThreads. See comment above.
                continue;
            }
            thread.thread = _startThread(thread.loopIteration, thread.threadName);
        }
        _mutex.unlock(); // Was locked in the before-fork handler
    }

    boost::thread ThreadSystem::_startThread(function<bool()> loopIteration, const string& threadName) {
        return boost::thread([loopIteration = std::move(loopIteration), threadName] {
            cpputils::set_thread_name(threadName.c_str());
            ThreadSystem::_runThread(loopIteration);
        });
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
            LOG(ERR, "LoopThread crashed: {}", e.what());
        } catch (...) {
            LOG(ERR, "LoopThread crashed");
        }
        //TODO We should remove the thread from _runningThreads here, not in stop().
    }
}
