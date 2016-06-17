#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPENOTREADABLEERROR_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPENOTREADABLEERROR_H

#include <stdexcept>

namespace cpputils {
    namespace process {
        class PipeNotReadableError final : public std::runtime_error {
        public:
            PipeNotReadableError(): std::runtime_error("Pipe not readable") {
            }
        };
    }
}

#endif
