#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_PERIODICTASK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_PERIODICTASK_H_

#include <functional>
#include <boost/thread.hpp>

namespace blockstore {
namespace caching {

class PeriodicTask {
public:
	PeriodicTask(std::function<void ()> task, double intervalSec);
	virtual ~PeriodicTask();

private:
  boost::thread _thread;
  std::function<void ()> _task;
  double _intervalSec;
};

}
}

#endif
