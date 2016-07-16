#include "WorkerThread.h"

using std::packaged_task;

namespace cpputils {

    WorkerThread::WorkerThread(ThreadsafeQueue<packaged_task<void ()>> *taskQueue)
        :_taskQueue(taskQueue), _thread(std::bind(&WorkerThread::_loopIteration, this)) {

        _thread.start();
    }

    bool WorkerThread::_loopIteration() {
        auto task = _taskQueue->waitAndPop();
        task();
        return true; // Run another iteration
    }

}
