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

    template<class LockName>
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
    template<class LockName>
    inline LockPool<LockName>::LockPool(): _lockedLocks(), _mutex(), _cv() {}

    template<class LockName>
    inline LockPool<LockName>::~LockPool() {
        ASSERT(_lockedLocks.size() == 0, "Still locks open");
    }

    template<class LockName>
    inline void LockPool<LockName>::lock(const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting) {
        ASSERT(lockToFreeWhileWaiting->owns_lock(), "Given lock must be locked");
        std::unique_lock<std::mutex> mutexLock(_mutex);
        // Order of threadsafe/unlocking is important and should be the same order as everywhere else to prevent deadlocks.
        // Since when entering the function, lockToFreeWhileWaiting is already locked and mutexLock is locked afterwards,
        // the condition variable should do it in the same order. We use combinedLock for this.
        CombinedLock combinedLock(lockToFreeWhileWaiting, &mutexLock);
        _lock(lockName, &combinedLock);
        ASSERT(mutexLock.owns_lock() && lockToFreeWhileWaiting->owns_lock(), "Locks haven't been correctly relocked");
    }

    template<class LockName>
    inline void LockPool<LockName>::lock(const LockName &lockName) {
        std::unique_lock<std::mutex> mutexLock(_mutex);
        _lock(lockName, &mutexLock);
        ASSERT(mutexLock.owns_lock(), "Lock hasn't been correctly relocked");
    }

    template<class LockName>
    template<class OuterLock>
    inline void LockPool<LockName>::_lock(const LockName &lockName, OuterLock *mutexLock) {
        if (!_canLock(lockName)) {
            _cv.wait(*mutexLock, [this, &lockName]{
              return _canLock(lockName);
            });
        }
        _lockedLocks.emplace_back(Locked {lockName, std::this_thread::get_id()});
    }

    template<class LockName>
    boost::optional<typename std::vector<typename LockPool<LockName>::Locked>::const_iterator> LockPool<LockName>::_findLock(const LockName &lockName) const {
        auto found = std::find_if(_lockedLocks.begin(), _lockedLocks.end(), [&lockName] (const Locked& entry) {return entry.lockName == lockName;});
        if (found == _lockedLocks.end()) {
          return boost::none;
        }
        return found;
    }

    template<class LockName>
    boost::optional<typename std::vector<typename LockPool<LockName>::Locked>::iterator> LockPool<LockName>::_findLock(const LockName &lockName) {
      auto found = const_cast<const LockPool*>(this)->_findLock(lockName);
      if (found == boost::none) {
        return boost::none;
      }
      return _lockedLocks.erase(*found, *found); // this doesn't actually erase anything but only transforms const_iterator into iterator
    }

    template<class LockName>
    inline bool LockPool<LockName>::_canLock(const LockName &lockName) const {
        auto lock = _findLock(lockName);
        if (lock == boost::none) {
            return true;
        }
        if ((*lock)->ownerThread == std::this_thread::get_id()) {
            throw std::runtime_error("Thread tried to get same lock twice from lock pool");
        }
        return false;
    }

    template<class LockName>
    inline void LockPool<LockName>::release(const LockName &lockName) {
        std::unique_lock<std::mutex> mutexLock(_mutex);
        auto found = _findLock(lockName);
        ASSERT(found != boost::none, "Lock given to release() was not locked");
        _lockedLocks.erase(*found);
        _cv.notify_all();
    }
}

#endif
