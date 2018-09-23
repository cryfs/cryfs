#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEREADER_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEREADER_H

#include "../../macros.h"
#include "PipeStreamEndpoint.h"
#include <cstdint>
#include <string>
#include "PipeNotReadableError.h"

namespace cpputils {
    namespace process {
        class PipeReader final {
        public:
            constexpr static const uint64_t MAX_READ_SIZE = 10 * 1024 * 1024;

            PipeReader(PipeDescriptor fd);
            PipeReader(PipeReader &&rhs) = default;

            std::string receive();

        private:
            PipeStreamEndpoint _stream;

            DISALLOW_COPY_AND_ASSIGN(PipeReader);
        };
    }
}

#endif
