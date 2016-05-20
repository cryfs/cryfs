#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_DAEMONIZE_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_DAEMONIZE_H

#include <functional>
#include "PipeFromChild.h"
#include "PipeToParent.h"

namespace cpputils {
    namespace process {
        /**
         * Fork a child process, set it up as a daemon process, and run childProgram inside it.
         *
         * The childProgram gets a PipeToParent, which it can use to notify the parent process
         * when the daemon is ready, or if there was an error during initialization.
         * After execution of the childProgram, the child process will exit and not return to the caller.
         * That means for example that no destructors of outside allocated objects are called.
         *
         * In the parent process, daemonize() returns to the caller and returns a PipeFromChild,
         * which can be used to wait for the ready signal from the child process.
         */
        PipeFromChild daemonize(std::function<void (PipeToParent *)> childProgram);
    }
}

#endif
