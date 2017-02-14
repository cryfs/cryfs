#pragma once
#ifndef MESSMER_CPPUTILS_NETWORK_CURLINITIALIZERRAII_HPP
#define MESSMER_CPPUTILS_NETWORK_CURLINITIALIZERRAII_HPP

#include <cpp-utils/macros.h>
#include <mutex>
#include <curl/curl.h>

namespace cpputils {
    // TODO Test

    // When the first object of this class is created, it will initialize curl using curl_global_init().
    // When the last object is destroyed, it will deinitialize curl using curl_global_cleanup().
    class CurlInitializerRAII final {
    public:
        CurlInitializerRAII();
        ~CurlInitializerRAII();
    private:
        static std::mutex _mutex;
        static uint32_t _refcount;

        DISALLOW_COPY_AND_ASSIGN(CurlInitializerRAII);
    };

}

#endif
