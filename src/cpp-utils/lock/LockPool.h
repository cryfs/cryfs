#pragma once
#ifndef MESSMER_CPPUTILS_LOCK_LOCKPOOL_H
#define MESSMER_CPPUTILS_LOCK_LOCKPOOL_H

#include <mutex>
#include <condition_variable>
#include <vector>
#include <algorithm>
#include "../assert/assert.h"
#include "../macros.h"
#include "CombinedLock.h"

//TODO Test
//TODO Rename package to synchronization
//TODO Rename to MutexPool
namespace cpputils {

    template<class LockName>
    class LockPool final {
    public:
        LockPool();
        ~LockPool();
        void lock(const LockName &lock, std::unique_lock<std::mutex> *lockToFreeWhileWaiting = nullptr);
        void release(const LockName &lock);

    private:
        bool _isLocked(const LockName &lock) const;

        std::vector<LockName> _lockedLocks;
        std::mutex _mutex;
        std::condition_variable_any _cv;

        DISALLOW_COPY_AND_ASSIGN(LockPool);
    };
    template<class LockName>
    inline LockPool<LockName>::LockPool(): _lockedLocks(), _mutex(), _cv() {}

    template<class LockName>
    inline LockPool<LockName>::~LockPool() {
        ASSERT(_lockedLocks.size() == 0, "Still locks open");
    }

    template<class LockName>
    inline void LockPool<LockName>::lock(const LockName &lock, std::unique_lock<std::mutex> *lockToFreeWhileWaiting) {
        std::unique_lock<std::mutex> mutexLock(_mutex); // TODO Is shared_lock enough here?
        if (_isLocked(lock)) {
            // Order of locking/unlocking is important and should be the same order as everywhere else to prevent deadlocks.
            // Since when entering the function, lockToFreeWhileWaiting is already locked and mutexLock is locked afterwards,
            // the condition variable should do it in the same order. We use combinedLock for this.
            CombinedLock combinedLock(lockToFreeWhileWaiting, &mutexLock);
            _cv.wait(combinedLock, [this, &lock]{
                return !_isLocked(lock);
            });
            ASSERT(mutexLock.owns_lock() && lockToFreeWhileWaiting->owns_lock(), "Locks haven't been correctly relocked");
        }
        _lockedLocks.push_back(lock);
    }

    template<class LockName>
    inline bool LockPool<LockName>::_isLocked(const LockName &lock) const {
        return std::find(_lockedLocks.begin(), _lockedLocks.end(), lock) != _lockedLocks.end();
    }

    template<class LockName>
    inline void LockPool<LockName>::release(const LockName &lock) {
        std::unique_lock<std::mutex> mutexLock(_mutex);
        auto found = std::find(_lockedLocks.begin(), _lockedLocks.end(), lock);
        ASSERT(found != _lockedLocks.end(), "Lock given to release() was not locked");
        _lockedLocks.erase(found);
        _cv.notify_all();
    }
}

#endif
