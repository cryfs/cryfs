#include "PeriodicTask.h"
#include <messmer/cpp-utils/logging/logging.h>

using std::function;
using std::endl;
using namespace cpputils::logging;

namespace blockstore {
namespace caching {

PeriodicTask::PeriodicTask(function<void ()> task, double intervalSec) : _thread(), _task(task), _intervalSec(intervalSec) {
  _thread = boost::thread([this]() {
    boost::chrono::nanoseconds interval((uint64_t)(UINT64_C(1000000000) * _intervalSec));
    try {
      while(true) {
        boost::this_thread::sleep_for(interval);
        _task();
      }
    } catch (const boost::thread_interrupted &e) {
      //Do nothing, exit thread.
    } catch (const std::exception &e) {
      LOG(ERROR) << "PeriodicTask crashed: " << e.what();
    } catch (...) {
      LOG(ERROR) << "PeriodicTask crashed";
    }
  });
}

PeriodicTask::~PeriodicTask() {
  _thread.interrupt();
  _thread.join();
}

}
}
