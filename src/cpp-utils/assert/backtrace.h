#pragma once
#ifndef MESSMER_CPPUTILS_ASSERT_BACKTRACE_H
#define MESSMER_CPPUTILS_ASSERT_BACKTRACE_H

#include <string>

namespace cpputils {
    std::string backtrace();
    void showBacktraceOnSigSegv();
}

#endif
