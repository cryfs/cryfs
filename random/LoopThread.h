#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_LOOPTHREAD_H
#define MESSMER_CPPUTILS_RANDOM_LOOPTHREAD_H

#include <boost/thread.hpp>

namespace cpputils {
    //TODO Test
    //TODO Move out of "random" folder into own library folder
    // Has to be final, because otherwise there could be a race condition where LoopThreadForkHandler calls a LoopThread
    // where the child class destructor already ran.
    class LoopThread final {
    public:
        LoopThread(std::function<void()> loopIteration);
        ~LoopThread();
        void start();
        void stop();

        void asyncStop();
        void waitUntilStopped();

    private:
        void main();
        boost::thread _thread;
        std::function<void()> _loopIteration;
    };
}

#endif
