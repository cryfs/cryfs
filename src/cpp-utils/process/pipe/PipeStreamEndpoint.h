#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPESTREAMENDPOINT_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPESTREAMENDPOINT_H

#include "../../macros.h"
#include "PipeDescriptor.h"
#include <cstdio>

namespace cpputils {
    namespace process {

        class PipeStreamEndpoint final {
        public:
            PipeStreamEndpoint(PipeDescriptor fd, const char *mode);
            PipeStreamEndpoint(PipeStreamEndpoint &&rhs);
            ~PipeStreamEndpoint();

            FILE *stream();

        private:
            PipeDescriptor _fd;
            FILE *_stream;

            DISALLOW_COPY_AND_ASSIGN(PipeStreamEndpoint);
        };
    }
}

#endif
