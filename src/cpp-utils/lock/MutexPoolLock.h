#pragma once
#ifndef MESSMER_CPPUTILS_LOCK_MUTEXPOOLLOCK_H
#define MESSMER_CPPUTILS_LOCK_MUTEXPOOLLOCK_H

#include "LockPool.h"

namespace cpputils {
    template<class LockName, bool Recursive = false>
    class MutexPoolLock final {
    public:
        MutexPoolLock(LockPool<LockName, Recursive> *pool, const LockName &lockName): _pool(pool), _lockName(lockName) {
            _pool->lock(_lockName);
        }

        MutexPoolLock(LockPool<LockName, Recursive> *pool, const LockName &lockName, std::unique_lock<std::mutex> *lockToFreeWhileWaiting)
                : _pool(pool), _lockName(lockName) {
            _pool->lock(_lockName, lockToFreeWhileWaiting);
        }
        
        MutexPoolLock(MutexPoolLock &&rhs) noexcept: _pool(rhs._pool), _lockName(std::move(rhs._lockName)) {
            rhs._pool = nullptr;
        }

        ~MutexPoolLock() {
            if (_pool != nullptr) {
                unlock();
            }
        }

        void unlock() {
            ASSERT(_pool != nullptr, "MutexPoolLock is not locked");
            _pool->release(_lockName);
            _pool = nullptr;
        }

    private:
        LockPool<LockName, Recursive> *_pool;
        LockName _lockName;
        
        DISALLOW_COPY_AND_ASSIGN(MutexPoolLock);
    };

    template<class LockName> using RecursiveMutexPoolLock = MutexPoolLock<LockName, true>;
}

#endif
