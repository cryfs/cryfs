#pragma once
#ifndef MESSMER_CPPUTILS_LOCK_CONDITIONBARRIER_H
#define MESSMER_CPPUTILS_LOCK_CONDITIONBARRIER_H

#include "../macros.h"
#include <mutex>

//TODO Test
//TODO Merge lock folder with thread folder

namespace cpputils {
    // Like a condition variable, but without spurious wakeups.
    // The waiting threads are only woken, when notify() is called.
    // After a call to release(), future calls to wait() will not block anymore.
    template<class Mutex, class ConditionVariable>
    class ConditionBarrier final {
    public:
        ConditionBarrier() :_mutex(), _cv(), _triggered(false) {
        }

        void wait() {
            std::unique_lock<Mutex> lock(_mutex);
            _cv.wait(lock, [this] {
                return _triggered;
            });
        }

        void release() {
            std::unique_lock<Mutex> lock(_mutex);
            _triggered = true;
            _cv.notify_all();
        }
    private:
        Mutex _mutex;
        ConditionVariable _cv;
        bool _triggered;

        DISALLOW_COPY_AND_ASSIGN(ConditionBarrier);
    };
}

#endif
