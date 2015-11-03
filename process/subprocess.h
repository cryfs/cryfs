#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H
#define MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H

#include <string>

namespace cpputils {
    //TODO Test
    class Subprocess {
    public:
        static std::string call(const std::string &command);
        static int callAndGetReturnCode(const std::string &command);
    private:
        static FILE* _call(const std::string &command);
    };
}

#endif
