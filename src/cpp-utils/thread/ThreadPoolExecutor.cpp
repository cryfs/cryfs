#include "ThreadPoolExecutor.h"

using std::vector;
using folly::Function;

namespace cpputils {

ThreadPoolExecutor::ThreadPoolExecutor(size_t numThreads)
: _tasks(), _executorThreads(_createExecutorThreads(numThreads)) {}

ThreadPoolExecutor::~ThreadPoolExecutor() {
    _tasks.waitUntilEmpty();
}

vector<LoopThread> ThreadPoolExecutor::_createExecutorThreads(size_t numThreads) {
    vector<LoopThread> result;
    result.reserve(numThreads);
    for (size_t i = 0; i < numThreads; ++i) {
        result.push_back(LoopThread([this] () {
            return _executorThreadIteration();
        }));
        result.back().start();
    }
    return result;
}

bool ThreadPoolExecutor::_executorThreadIteration() {
    auto task = _tasks.pop();
    task();
    return true;
}

void ThreadPoolExecutor::execute(Function<void ()> task) {
    _tasks.push(std::move(task));
}

}
