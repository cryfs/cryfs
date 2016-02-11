#include "PeriodicTask.h"
#include <cpp-utils/logging/logging.h>

using std::function;
using std::endl;
using namespace cpputils::logging;

namespace blockstore {
namespace caching {

PeriodicTask::PeriodicTask(function<void ()> task, double intervalSec) :
        _task(task),
        _interval((uint64_t)(UINT64_C(1000000000) * intervalSec)),
        _thread(std::bind(&PeriodicTask::_loopIteration, this)) {
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
