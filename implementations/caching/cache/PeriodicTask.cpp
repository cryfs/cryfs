#include "PeriodicTask.h"

using std::function;
using std::cerr;
using std::endl;

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
      //TODO Think about logging
      cerr << "PeriodicTask crashed: " << e.what() << endl;
    } catch (...) {
      //TODO Think about logging
      cerr << "PeriodicTask crashed" << endl;
    }
  });
}

PeriodicTask::~PeriodicTask() {
  _thread.interrupt();
  _thread.join();
}

}
}
