#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEDESCRIPTOR_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEDESCRIPTOR_H

#include "../../macros.h"

namespace cpputils {
    namespace process {

        class PipeDescriptor final {
        public:
            PipeDescriptor();
            PipeDescriptor(int fd);
            PipeDescriptor(PipeDescriptor &&rhs);
            ~PipeDescriptor();
            PipeDescriptor &operator=(PipeDescriptor &&rhs);

            bool valid() const;
            int fd();
            void close();

            // This is called, when the underlying file descriptor was closed by another object, for example using fclose() on a created FILE object.
            void wasClosedOutside();

        private:
            int _fd;

            void _destruct();

            DISALLOW_COPY_AND_ASSIGN(PipeDescriptor);
        };
    }
}


#endif
