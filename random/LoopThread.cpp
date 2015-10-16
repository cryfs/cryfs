#include "LoopThread.h"

namespace cpputils {

    LoopThread::LoopThread(): _thread() {}

    LoopThread::~LoopThread() {
        stop();
    }

    void LoopThread::start() {
        _thread = boost::thread(std::bind(&LoopThread::main, this));
    }

    void LoopThread::stop() {
        _thread.interrupt();
        _thread.join();
    }

    void LoopThread::main() {
        try {
            while(true) {
                loopIteration();
            }
        } catch (const boost::thread_interrupted &e) {
            //Do nothing, exit thread.
        } catch (const std::exception &e) {
            //TODO Think about logging
            std::cerr << "LoopThread crashed: " << e.what() << std::endl;
        } catch (...) {
            //TODO Think about logging
            std::cerr << "LoopThread crashed" << std::endl;
        }
    }
}