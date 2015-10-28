#include "LoopThread.h"
#include "../logging/logging.h"
#include "LoopThreadForkHandler.h"

using namespace cpputils::logging;
using std::function;

namespace cpputils {

    LoopThread::LoopThread(function<void()> loopIteration): _thread(), _loopIteration(loopIteration) {
        LoopThreadForkHandler::singleton().add(this);
    }

    LoopThread::~LoopThread() {
        LoopThreadForkHandler::singleton().remove(this);
        stop();
    }

    void LoopThread::start() {
        _thread = boost::thread(std::bind(&LoopThread::main, this));
    }

    void LoopThread::stop() {
        asyncStop();
        waitUntilStopped();
    }

    void LoopThread::asyncStop() {
        _thread.interrupt();
    }
    void LoopThread::waitUntilStopped() {
        _thread.join();
    }

    void LoopThread::main() {
        try {
            while(true) {
                _loopIteration();
            }
        } catch (const boost::thread_interrupted &e) {
            //Do nothing, exit thread.
        } catch (const std::exception &e) {
            //TODO Think about logging
            LOG(ERROR) << "LoopThread crashed: " << e.what();
        } catch (...) {
            //TODO Think about logging
            LOG(ERROR) << "LoopThread crashed";
        }
    }
}
