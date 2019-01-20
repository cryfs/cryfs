#if !defined(_MSC_VER)

#include "debugging.h"
#include <stdexcept>
#include <thread>
#include <pthread.h>
#include <cpp-utils/assert/assert.h>

namespace cpputils {

namespace {
constexpr size_t MAX_NAME_LEN = 16; // this length includes the terminating null character at the end
}

void set_thread_name(const char* name) {
  std::string name_(name);
  if (name_.size() > MAX_NAME_LEN - 1) {
    name_.resize(MAX_NAME_LEN - 1);
  }
#if defined(__APPLE__)
  int result = pthread_setname_np(name_.c_str());
#else
  int result = pthread_setname_np(pthread_self(), name_.c_str());
#endif
  if (0 != result) {
    throw std::runtime_error("Error setting thread name with pthread_setname_np. Code: " + std::to_string(result));
  }
}

namespace {
std::string get_thread_name(pthread_t thread) {
  char name[MAX_NAME_LEN];
  int result = pthread_getname_np(thread, name, MAX_NAME_LEN);
  if (0 != result) {
    throw std::runtime_error("Error getting thread name with pthread_getname_np. Code: " + std::to_string(result));
  }
  // pthread_getname_np returns a null terminated string with maximum 16 bytes.
  // but just to be safe against a buggy implementation, let's set the last byte to zero.
  name[MAX_NAME_LEN - 1] = '\0';
  return name;
}
}

std::string get_thread_name() {
  return get_thread_name(pthread_self());
}

std::string get_thread_name(std::thread* thread) {
  ASSERT(thread->joinable(), "Thread not running");
  return get_thread_name(thread->native_handle());
}

}

#endif
