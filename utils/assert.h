#ifndef CRYFS_ASSERT_H
#define CRYFS_ASSERT_H

#include "IOException.h"
#include <iostream>
#include <string>

namespace fspp {
    namespace _assert {
        inline std::string format(const char *expr, const char *message, const char *file, int line) {
            return std::string()+"Assertion ["+expr+"] failed in "+file+":"+std::to_string(line)+": "+message;
        }

        inline void assert_fail_release(const char *expr, const char *message, const char *file, int line) {
            throw IOException(format(expr, message, file, line));
        }

        inline void assert_fail_debug(const char *expr, const char *message, const char *file, int line) {
            std::cerr << format(expr, message, file, line) << std::endl;
            abort();
        }
    }
}

#ifdef NDEBUG
//TODO Check whether disabling assertions in prod affects speed.
# define fspp_assert(expr, msg) (void)((expr) || (fspp::_assert::assert_fail_release(#expr, msg, __FILE__, __LINE__),0))
#else
# define fspp_assert(expr, msg) (void)((expr) || (fspp::_assert::assert_fail_debug(#expr, msg, __FILE__, __LINE__),0))
#endif

#endif
