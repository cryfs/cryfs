#include "PeriodicTask.h"
#include <cpp-utils/logging/logging.h>

using std::function;
using namespace cpputils::logging;

namespace blockstore {
namespace caching {

PeriodicTask::PeriodicTask(function<void ()> task, double intervalSec, std::string threadName) :
        _task(task),
        _interval(static_cast<uint64_t>(UINT64_C(1000000000) * intervalSec)),
        _thread(std::bind(&PeriodicTask::_loopIteration, this), std::move(threadName)) {
    _thread.start();
}

bool PeriodicTask::_loopIteration() {
  //Has to be boost::this_thread::sleep_for and not std::this_thread::sleep_for, because it has to be interruptible.
  //LoopThread will interrupt this method if it has to be restarted.
  boost::this_thread::sleep_for(_interval);
  _task();
  return true; // Run another iteration (don't terminate thread)
}

}
}
