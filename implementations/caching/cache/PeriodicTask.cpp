#include "PeriodicTask.h"
#include <messmer/cpp-utils/logging/logging.h>

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

void PeriodicTask::_loopIteration() {
  try {
    boost::this_thread::sleep_for(_interval);
    _task();
  } catch (const std::exception &e) {
    LOG(ERROR) << "PeriodicTask crashed: " << e.what();
    throw;
  } catch (...) {
    LOG(ERROR) << "PeriodicTask crashed";
    throw;
  }
}

}
}
