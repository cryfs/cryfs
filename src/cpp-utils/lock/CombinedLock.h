#ifndef MESSMER_CPPUTILS_LOCK_COMBINEDLOCK_H
#define MESSMER_CPPUTILS_LOCK_COMBINEDLOCK_H

#include "../macros.h"

namespace cpputils {

    /**
     * This class is used to combine multiple locks into one, taking care that they are locked/unlocked
     * in the order they were given to the constructor.
     */
    class CombinedLock final {
    public:
        CombinedLock(std::unique_lock<std::mutex> *outer, std::unique_lock<std::mutex> *inner)
                : _outer(outer), _inner(inner) {
        }

        void lock() {
            _outer->lock();
            _inner->lock();
        }

        void unlock() {
            _inner->unlock();
            _outer->unlock();
        }

    private:
        std::unique_lock<std::mutex> *_outer;
        std::unique_lock<std::mutex> *_inner;

        DISALLOW_COPY_AND_ASSIGN(CombinedLock);
    };
}

#endif
