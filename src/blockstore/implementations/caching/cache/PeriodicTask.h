#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_PERIODICTASK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_PERIODICTASK_H_

#include <functional>
#include <cpp-utils/thread/LoopThread.h>
#include <boost/chrono.hpp>

namespace blockstore {
namespace caching {

class PeriodicTask final {
public:
	PeriodicTask(std::function<void ()> task, double intervalSec, std::string threadName);

private:
  bool _loopIteration();

  std::function<void()> _task;
  boost::chrono::nanoseconds _interval;

  //This member has to be last, so the thread is destructed first. Otherwise the thread might access elements from a
  //partly destructed PeriodicTask.
  cpputils::LoopThread _thread;

  DISALLOW_COPY_AND_ASSIGN(PeriodicTask);
};

}
}

#endif
