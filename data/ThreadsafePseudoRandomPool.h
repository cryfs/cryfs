#pragma once
#ifndef MESSMER_CPPUTILS_DATA_THREADSAFEPSEUDORANDOMPOOL_H
#define MESSMER_CPPUTILS_DATA_THREADSAFEPSEUDORANDOMPOOL_H

#include "../macros.h"
#include <mutex>
#include <cryptopp/cryptopp/osrng.h>

namespace cpputils {

    //TODO Create more complete random library around CryptoPP (also offering OS_Random for example) and use it in FixedSizeDate::CreateRandom()/CreateOSRandom()
    //TODO Store a static CryptoPP::AutoSeededRandomPool (or multiple ones) and make constructor of
    //     ThreadsafeRandomPool() be lightweight (i.e. not do seeding), so it can be called on each callsite.
    //     Alternatively, use a singleton factory.

    //TODO Test
    class ThreadsafePseudoRandomPool {
    public:
        ThreadsafePseudoRandomPool() { }

        void GenerateBlock(byte *data, size_t size) {
            // TODO Provide multiple randomPools for parallelity instead of locking the only available one
            std::unique_lock <std::mutex> lock(_mutex);
            _pool.GenerateBlock(data, size);
        }

    private:
        //TODO Make seeding use blocking=true (aka /dev/random instead of /dev/urandom) or offer a configuration option?
        CryptoPP::AutoSeededRandomPool _pool;
        std::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(ThreadsafePseudoRandomPool);
    };

}

#endif
