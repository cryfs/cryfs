#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_LOOPTHREAD_H
#define MESSMER_CPPUTILS_RANDOM_LOOPTHREAD_H

#include <boost/thread.hpp>

namespace cpputils {
    //TODO Test
    //TODO Move out of "random" folder into own library folder
    class LoopThread {
    public:
        LoopThread();
        virtual ~LoopThread();
        void start();
        void stop();

        virtual void loopIteration() = 0;

    private:
        void main();
        boost::thread _thread;
    };
}

#endif
