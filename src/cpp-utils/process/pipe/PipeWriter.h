#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEWRITER_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEWRITER_H

#include "../../macros.h"
#include "PipeStreamEndpoint.h"
#include <string>

namespace cpputils {
    namespace process {

        class PipeWriter final {
        public:
            PipeWriter(PipeDescriptor fd);
            PipeWriter(PipeWriter &&rhs) = default;

            void write(const std::string &str);

        private:
            PipeStreamEndpoint _stream;

            DISALLOW_COPY_AND_ASSIGN(PipeWriter);
        };
    }
}


#endif
