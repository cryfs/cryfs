#pragma once
#ifndef MESSMER_CPPUTILS_LOCK_CONDITIONBARRIER_H
#define MESSMER_CPPUTILS_LOCK_CONDITIONBARRIER_H

#include <mutex>
#include <condition_variable>
#include "../macros.h"

//TODO Test
//TODO Merge lock folder with thread folder

namespace cpputils {
    // Like a condition variable, but without spurious wakeups.
    // The waiting threads are only woken, when notify() is called.
    // After a call to release(), future calls to wait() will not block anymore.
    class ConditionBarrier final {
    public:
        ConditionBarrier() :_mutex(), _cv(), _triggered(false) {
        }

        void wait() {
            std::unique_lock<std::mutex> lock(_mutex);
            _cv.wait(lock, [this] {
                return _triggered;
            });
        }

        void release() {
            const std::unique_lock<std::mutex> lock(_mutex);
            _triggered = true;
            _cv.notify_all();
        }
    private:
        std::mutex _mutex;
        std::condition_variable _cv;
        bool _triggered;

        DISALLOW_COPY_AND_ASSIGN(ConditionBarrier);
    };
}

#endif
