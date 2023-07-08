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
        void lock(const LockName &lockName);
        void lock(const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting);
        void release(const LockName &lockName);

    private:
        bool _isLocked(const LockName &lockName) const;
        template<class OuterLock> void _lock(const LockName &lockName, OuterLock *lockToFreeWhileWaiting);

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
    inline void LockPool<LockName>::lock(const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting) {
        ASSERT(lockToFreeWhileWaiting->owns_lock(), "Given lock must be locked");
        std::unique_lock<std::mutex> mutexLock(_mutex); // TODO Is shared_lock enough here?
        // Order of locking/unlocking is important and should be the same order as everywhere else to prevent deadlocks.
        // Since when entering the function, lockToFreeWhileWaiting is already locked and mutexLock is locked afterwards,
        // the condition variable should do it in the same order. We use combinedLock for this.
        CombinedLock combinedLock(lockToFreeWhileWaiting, &mutexLock);
        _lock(lockName, &combinedLock);
        ASSERT(mutexLock.owns_lock() && lockToFreeWhileWaiting->owns_lock(), "Locks haven't been correctly relocked");
    }

    template<class LockName>
    inline void LockPool<LockName>::lock(const LockName &lockName) {
        std::unique_lock<std::mutex> mutexLock(_mutex); // TODO Is shared_lock enough here?
        _lock(lockName, &mutexLock);
        ASSERT(mutexLock.owns_lock(), "Lock hasn't been correctly relocked");
    }

    template<class LockName>
    template<class OuterLock>
    inline void LockPool<LockName>::_lock(const LockName &lockName, OuterLock *mutexLock) {
        if (_isLocked(lockName)) {
            _cv.wait(*mutexLock, [this, &lockName]{
                return !_isLocked(lockName);
            });
        }
        _lockedLocks.push_back(lockName);
    }

    template<class LockName>
    inline bool LockPool<LockName>::_isLocked(const LockName &lockName) const {
        return std::find(_lockedLocks.begin(), _lockedLocks.end(), lockName) != _lockedLocks.end();
    }

    template<class LockName>
    inline void LockPool<LockName>::release(const LockName &lockName) {
        const std::unique_lock<std::mutex> mutexLock(_mutex);
        auto found = std::find(_lockedLocks.begin(), _lockedLocks.end(), lockName);
        ASSERT(found != _lockedLocks.end(), "Lock given to release() was not locked");
        _lockedLocks.erase(found);
        _cv.notify_all();
    }
}

#endif
