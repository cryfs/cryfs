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
#include <boost/optional.hpp>

//TODO Test
//TODO Rename package to synchronization
//TODO Rename to MutexPool
namespace cpputils {

    template<class LockName, bool Recursive = false>
    class LockPool final {
    public:
        LockPool();
        ~LockPool();
        void lock(const LockName &lockName);
        void lock(const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting);
        void release(const LockName &lockName);

    private:
        struct Locked final {
            LockName lockName;
            std::thread::id ownerThread;
            size_t lockCount;
        };

        bool _canLock(const LockName &lockName) const;
        template<class OuterLock> void _lock(const LockName &lockName, OuterLock *lockToFreeWhileWaiting);
        boost::optional<typename std::vector<Locked>::const_iterator> _findLock(const LockName &lockName) const;
        boost::optional<typename std::vector<Locked>::iterator> _findLock(const LockName &lockName);

        std::vector<Locked> _lockedLocks;
        std::mutex _mutex;
        std::condition_variable_any _cv;

        DISALLOW_COPY_AND_ASSIGN(LockPool);
    };
    template<class LockName, bool Recursive>
    inline LockPool<LockName, Recursive>::LockPool(): _lockedLocks(), _mutex(), _cv() {}

    template<class LockName, bool Recursive>
    inline LockPool<LockName, Recursive>::~LockPool() {
        ASSERT(_lockedLocks.size() == 0, "Still locks open");
    }

    template<class LockName, bool Recursive>
    inline void LockPool<LockName, Recursive>::lock(const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting) {
        ASSERT(lockToFreeWhileWaiting->owns_lock(), "Given lock must be locked");
        std::unique_lock<std::mutex> mutexLock(_mutex); // TODO Is shared_lock enough here?
        // Order of threadsafe/unlocking is important and should be the same order as everywhere else to prevent deadlocks.
        // Since when entering the function, lockToFreeWhileWaiting is already locked and mutexLock is locked afterwards,
        // the condition variable should do it in the same order. We use combinedLock for this.
        CombinedLock combinedLock(lockToFreeWhileWaiting, &mutexLock);
        _lock(lockName, &combinedLock);
        ASSERT(mutexLock.owns_lock() && lockToFreeWhileWaiting->owns_lock(), "Locks haven't been correctly relocked");
    }

    template<class LockName, bool Recursive>
    inline void LockPool<LockName, Recursive>::lock(const LockName &lockName) {
        std::unique_lock<std::mutex> mutexLock(_mutex); // TODO Is shared_lock enough here?
        _lock(lockName, &mutexLock);
        ASSERT(mutexLock.owns_lock(), "Lock hasn't been correctly relocked");
    }

    template<class LockName, bool Recursive>
    template<class OuterLock>
    inline void LockPool<LockName, Recursive>::_lock(const LockName &lockName, OuterLock *mutexLock) {
        if (!_canLock(lockName)) {
            _cv.wait(*mutexLock, [this, &lockName]{
                return _canLock(lockName);
            });
        }
        // TODO Performance: Why run find again if _canLock above already did?
        auto found = _findLock(lockName);
        if (found == boost::none) {
            _lockedLocks.push_back({.lockName = lockName, .ownerThread = std::this_thread::get_id(), .lockCount = 1});
        } else {
            (*found)->lockCount += 1;
        }
    }

    template<class LockName, bool Recursive>
    boost::optional<typename std::vector<typename LockPool<LockName, Recursive>::Locked>::const_iterator> LockPool<LockName, Recursive>::_findLock(const LockName &lockName) const {
        auto found = std::find_if(_lockedLocks.begin(), _lockedLocks.end(), [&lockName] (const auto& entry) {return entry.lockName == lockName;});
        if (found == _lockedLocks.end()) {
            return boost::none;
        }
        return found;
    }

    template<class LockName, bool Recursive>
    boost::optional<typename std::vector<typename LockPool<LockName, Recursive>::Locked>::iterator> LockPool<LockName, Recursive>::_findLock(const LockName &lockName) {
        auto found = const_cast<const LockPool*>(this)->_findLock(lockName);
        if (found == boost::none) {
            return boost::none;
        }
        return _lockedLocks.erase(*found, *found); // this doesn't actually erase anything but only transforms const_iterator into iterator
    }

    template<class LockName, bool Recursive>
    inline bool LockPool<LockName, Recursive>::_canLock(const LockName &lockName) const {
        auto lock = _findLock(lockName);
        if (lock == boost::none) {
            return true;
        }
        if ((*lock)->ownerThread == std::this_thread::get_id()) {
            if (Recursive) {
                // recursive locks can be locked multiple times in the same thread
                return true;
            } else {
                throw std::runtime_error("Thread tried to get same lock twice from non-recursive lock pool");
            }
        }
        return false;
    }

    template<class LockName, bool Recursive>
    inline void LockPool<LockName, Recursive>::release(const LockName &lockName) {
        std::unique_lock<std::mutex> mutexLock(_mutex);
        auto found = _findLock(lockName);
        ASSERT(found != boost::none, "Lock given to release() was not locked");
        ASSERT((*found)->lockCount >= 1, "Invalid state");
        if ((*found)->lockCount == 1) {
            _lockedLocks.erase(*found);
        } else {
            (*found)->lockCount -= 1;
        }
        _cv.notify_all();
    }

    template<class LockName> using RecursiveLockPool = LockPool<LockName, true>;
}

#endif
