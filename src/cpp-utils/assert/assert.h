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
#include "backtrace.h"
#include "../logging/logging.h"

namespace cpputils {
    namespace _assert {
        inline std::string format(const char *expr, const std::string &message, const char *file, int line) {
            std::string result = std::string()+"Assertion ["+expr+"] failed in "+file+":"+std::to_string(line)+": "+message+"\n\n" + backtrace();
            return result;
        }

        inline void assert_fail_release [[noreturn]] (const char *expr, const std::string &message, const char *file, int line) {
            auto msg = format(expr, message, file, line);
            using namespace logging;
            LOG(ERR, msg);
            throw AssertFailed(msg);
        }

        inline void assert_fail_debug [[noreturn]] (const char *expr, const std::string &message, const char *file, int line) {
            using namespace logging;
            LOG(ERR, format(expr, message, file, line));
            abort();
        }
    }
}

#ifdef NDEBUG
//TODO Check whether disabling assertions in prod affects speed.
# define ASSERT(expr, msg) (void)((expr) || (cpputils::_assert::assert_fail_release(#expr, msg, __FILE__, __LINE__),0))
#else
# define ASSERT(expr, msg) (void)((expr) || (cpputils::_assert::assert_fail_debug(#expr, msg, __FILE__, __LINE__),0))
#endif

#endif
