#pragma once
#ifndef MESSMER_CPPUTILS_ASSERT_ASSERT_H
#define MESSMER_CPPUTILS_ASSERT_ASSERT_H

/**
 * This implements an ASSERT(expr, msg) macro.
 * In a debug build, it will crash and halt the program on an assert failure.
 * In a release build, it will throw an AssertFailed exception instead, which can then be caught.
 */

#include "AssertFailed.h"
#include <iostream>
#include <thread>
#include "backtrace.h"
#include "../logging/logging.h"

namespace cpputils {
    namespace _assert {
        struct DisableAbortOnFailedAssertionRAII final {
            explicit DisableAbortOnFailedAssertionRAII()
            : thread_id_(std::this_thread::get_id()) {
                ++num_instances_;
            }

            ~DisableAbortOnFailedAssertionRAII() {
                if (thread_id_ != std::this_thread::get_id()) {
                    using namespace logging;
                    LOG(ERR, "DisableAbortOnFailedAssertionRAII instance must be destructed in the same thread that created it");
                }
                --num_instances_;
            }

            static int num_instances() {
                return num_instances_;
            }

        private:
            static thread_local int num_instances_; // initialized to zero in assert.cpp

            std::thread::id thread_id_;
        };

        inline std::string format(const char *expr, const std::string &message, const char *file, int line) {
            std::string result = std::string()+"Assertion ["+expr+"] failed in "+file+":"+std::to_string(line)+": "+message+"\n\n" + backtrace();
            return result;
        }

        inline void assert_fail_release [[noreturn]] (const char *expr, const std::string &message, const char *file, int line) {
            using namespace logging;
            auto msg = format(expr, message, file, line);
            LOG(ERR, msg);
            throw AssertFailed(msg);
        }

        inline void assert_fail_debug [[noreturn]] (const char *expr, const std::string &message, const char *file, int line) {
            using namespace logging;
            auto msg = format(expr, message, file, line);
            LOG(ERR, msg);
            if (DisableAbortOnFailedAssertionRAII::num_instances() > 0) {
                throw AssertFailed(msg);
            } else {
                abort();
            }
        }
    }
}

#ifdef NDEBUG
    //TODO Check whether disabling assertions in prod affects speed.
    #define ASSERT(expr, msg) (void)((expr) || (cpputils::_assert::assert_fail_release(#expr, msg, __FILE__, __LINE__),0))
#else
    #define ASSERT(expr, msg) (void)((expr) || (cpputils::_assert::assert_fail_debug(#expr, msg, __FILE__, __LINE__),0))
#endif

#endif
