#pragma once
#ifndef MESSMER_CPPUTILS_ASSERT_BACKTRACE_H
#define MESSMER_CPPUTILS_ASSERT_BACKTRACE_H

#include <string>

namespace cpputils {
    std::string backtrace();

    //TODO Refactor (for example: RAII or at least try{}finally{} instead of  free())
    //TODO Use the following? https://github.com/bombela/backward-cpp
    void showBacktraceOnCrash();
}

#endif
