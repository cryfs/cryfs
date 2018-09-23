#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEBUILDER_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEBUILDER_H

#include "../../macros.h"
#include "PipeReader.h"
#include "PipeWriter.h"

namespace cpputils {
    namespace process {
        class PipeBuilder final {
        public:
            PipeBuilder();

            void closeReader();
            void closeWriter();

            PipeReader reader();
            PipeWriter writer();

        private:
            PipeDescriptor _readFd;
            PipeDescriptor _writeFd;

            DISALLOW_COPY_AND_ASSIGN(PipeBuilder);
        };
    }
}

#endif
