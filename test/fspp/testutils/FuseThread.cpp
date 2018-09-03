#include <sys/types.h>
#include "FuseThread.h"
#include <csignal>
#include <cpp-utils/assert/assert.h>
#include "fspp/fuse/Fuse.h"

using boost::thread;
using boost::chrono::seconds;
using std::string;
using std::vector;
namespace bf = boost::filesystem;

using fspp::fuse::Fuse;

FuseThread::FuseThread(Fuse *fuse)
  :_fuse(fuse), _child() {
}

void FuseThread::start(const bf::path &mountDir, const vector<string> &fuseOptions) {
  _child = thread([this, mountDir, fuseOptions] () {
    _fuse->run(mountDir, fuseOptions);
  });
  //Wait until it is running (busy waiting is simple and doesn't hurt much here)
  while(!_fuse->running()) {}
#ifdef __APPLE__
  // On Mac OS X, _fuse->running() returns true too early, because osxfuse calls init() when it's not ready yet. Give it a bit time.
  std::this_thread::sleep_for(std::chrono::milliseconds(200));
#endif
}

namespace {
#if !defined(_MSC_VER)
void kill_thread(boost::thread* thread) {
	if (0 != pthread_kill(thread->native_handle(), SIGINT)) {
		throw std::runtime_error("Error sending stop signal");
	}
}
#else
void kill_thread(boost::thread* thread) {
	if (0 == TerminateThread(thread->native_handle(), 0)) {
		throw std::runtime_error("Error sending stop signal");
	}
}
#endif
}

void FuseThread::stop() {
  kill_thread(&_child);
  bool thread_stopped = _child.try_join_for(seconds(10));
  ASSERT(thread_stopped, "FuseThread could not be stopped");
  //Wait until it is properly shutdown (busy waiting is simple and doesn't hurt much here)
  while (_fuse->running()) {}
}
