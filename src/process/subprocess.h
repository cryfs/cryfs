#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H
#define MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H

#include <string>
#include "../macros.h"

namespace cpputils {
    //TODO Test
    class Subprocess final {
    public:
        static std::string call(const std::string &command);
        static int callAndGetReturnCode(const std::string &command);
    private:
        static FILE* _call(const std::string &command);

        DISALLOW_COPY_AND_ASSIGN(Subprocess);
    };
}

#endif
