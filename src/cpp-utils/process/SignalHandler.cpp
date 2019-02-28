#include "SignalHandler.h"

#if !defined(_MSC_VER)

namespace cpputils {
namespace detail {
namespace {

std::atomic<bool> already_checked_for_libunwind_bug(false);

}

sigset_t _sigemptyset() {
    sigset_t result;
    int error = sigemptyset(&result);
    if (0 != error) {
        throw std::runtime_error("Error calling sigemptyset. Errno: " + std::to_string(errno));
    }
    return result;
}

void _sigmask(sigset_t* new_value, sigset_t* old_value) {
    int error = pthread_sigmask(SIG_SETMASK, new_value, old_value);
    if (0 != error) {
        throw std::runtime_error("Error calling pthread_sigmask. Errno: " + std::to_string(errno));
    }
}

// Check that we're not running into http://savannah.nongnu.org/bugs/?43752
void check_against_libunwind_bug() {
    if (already_checked_for_libunwind_bug.exchange(true)) {
        return;
    }

    // set new signal handler
    sigset_t old_value = _sigemptyset();
    sigset_t new_value = _sigemptyset();

    _sigmask(&new_value, &old_value);

    sigset_t before_exception = _sigemptyset();
    _sigmask(nullptr, &before_exception);

    // throw an exception
    try {
        throw std::runtime_error("Some exception");
    } catch (const std::exception &e) {}

    sigset_t after_exception = _sigemptyset();
    _sigmask(nullptr, &after_exception);

    // reset to old signal handler
    _sigmask(&old_value, nullptr);

    // check that the exception didn't screw up the signal mask
    if (0 != std::memcmp(&before_exception, &after_exception, sizeof(sigset_t))) {  // NOLINT(cppcoreguidelines-pro-type-union-access)
        ASSERT(false,
               "Throwing an exception screwed up the signal mask. You likely ran into this bug: http://savannah.nongnu.org/bugs/?43752 . Please build CryFS against a newer version of libunwind or build libunwind with --disable-cxx-exceptions.");
    }
}


}
}

#endif
