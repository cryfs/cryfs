#if !defined(_MSC_VER)

#include "debugging.h"
#include <stdexcept>
#include <thread>
#include <pthread.h>
#include <cpp-utils/assert/assert.h>
#if !(defined(__GLIBC__) || defined(__APPLE__))
// for pthread_getname_np_gcompat
#include <errno.h> // errno
#include <fcntl.h> // O_CLOEXEC, O_RDONLY
#include <unistd.h> // open, read
#include <boost/filesystem/path.hpp>
#include <sys/types.h>
#endif

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
#if !(defined(__GLIBC__) || defined(__APPLE__))

struct OpenFileRAII final {
    explicit OpenFileRAII(const char* filename) : fd(::open(filename, O_RDONLY | O_CLOEXEC)) {}
    ~OpenFileRAII() {
        int result = close(fd);
        if (result != 0) {
            throw std::runtime_error("Error closing file. Errno: " + std::to_string(errno));
        }
    }

    int fd;
};

// get the name of a thread
int pthread_getname_np_gcompat(pthread_t thread, char *name, size_t len) {
  ASSERT(thread == pthread_self(), "On musl systems, it's only supported to get the name of the current thread.");

  auto file = OpenFileRAII("/proc/thread-self/comm");
  ssize_t n;
  if (file.fd < 0)
    return errno;
  n = read(file.fd, name, len);
  if (n < 0)
    return errno;
  // if the trailing newline was not read, the buffer was too small
  if (n == 0 || name[n - 1] != '\n')
    return ERANGE;
  // null-terminate string
  name[n - 1] = '\0';
  return 0;
}
#endif

std::string get_thread_name(pthread_t thread) {
  std::array<char, MAX_NAME_LEN> name{};
#if defined(__GLIBC__) || defined(__APPLE__)
  int result = pthread_getname_np(thread, name.data(), MAX_NAME_LEN);
#else
  int result = pthread_getname_np_gcompat(thread, name.data(), MAX_NAME_LEN);
#endif
  if (0 != result) {
    throw std::runtime_error("Error getting thread name with pthread_getname_np. Code: " + std::to_string(result));
  }
  // pthread_getname_np returns a null terminated string with maximum 16 bytes.
  // but just to be safe against a buggy implementation, let's set the last byte to zero.
  name[MAX_NAME_LEN - 1] = '\0';
  return name.data();
}

}

std::string get_thread_name() {
  return get_thread_name(pthread_self());
}

#if defined(__GLIBC__) || defined(__APPLE__)
std::string get_thread_name(std::thread* thread) {
  ASSERT(thread->joinable(), "Thread not running");
  return get_thread_name(thread->native_handle());
}
#endif

}

#endif
