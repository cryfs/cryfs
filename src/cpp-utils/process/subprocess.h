#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H
#define MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H

#include <string>
#include <stdexcept>
#include "../macros.h"

namespace cpputils {
    struct SubprocessResult final {
        std::string output;
        int exitcode;
    };

    struct SubprocessError final : public std::runtime_error {
        SubprocessError(std::string msg): std::runtime_error(std::move(msg)) {}
    };

    //TODO Test
    class Subprocess final {
    public:
        static SubprocessResult call(const std::string &command);
        static SubprocessResult check_call(const std::string &command);
    private:

        DISALLOW_COPY_AND_ASSIGN(Subprocess);
    };
}

#endif
