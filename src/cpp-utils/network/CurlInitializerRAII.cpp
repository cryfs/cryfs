#include "CurlInitializerRAII.h"

using std::mutex;
using std::unique_lock;

namespace cpputils {

mutex CurlInitializerRAII::_mutex;
uint32_t CurlInitializerRAII::_refcount = 0;

CurlInitializerRAII::CurlInitializerRAII() {
    unique_lock<mutex> lock(_mutex);
    if (0 == _refcount) {
        curl_global_init(CURL_GLOBAL_ALL);
    }
    _refcount += 1;
}

CurlInitializerRAII::~CurlInitializerRAII() {
    unique_lock<mutex> lock(_mutex);
    _refcount -= 1;
    if (0 == _refcount) {
        curl_global_cleanup();
    }
}

}
