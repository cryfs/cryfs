#include "daemonize.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <syslog.h>
#include <string.h>
#include <iostream>
#include "../pipe/PipeBuilder.h"
#include "cpp-utils/logging/logging.h"

using namespace cpputils::logging;

using std::function;

namespace cpputils {
namespace process {


PipeFromChild daemonize(function<void (PipeToParent *)> childProgram) {
    PipeBuilder pipe;

    pid_t pid = fork();
    if (pid < 0) {
        throw std::runtime_error("fork() failed.");
    }
    if (pid > 0) {
        // We're the parent process.
        pipe.closeWriter();
        return PipeFromChild(pipe.reader());
    }

    PipeToParent pipeToParent(pipe.writer());

    // We're the child process.
    pipe.closeReader();

    umask(0);

    // Create a new SID for the child process
    pid_t sid = setsid();
    if (sid < 0) {
        pipeToParent.notifyError("Failed to get SID for pipe process");
        exit(EXIT_FAILURE);
    }

    // Change the current working directory to a directory that's always existing
    if ((chdir("/")) < 0) {
        pipeToParent.notifyError("Failed to change working directory for pipe process");
        exit(EXIT_FAILURE);
    }

    // Close out the standard file descriptors. The process can't use them anyhow.
    close(STDIN_FILENO);
    close(STDOUT_FILENO);
    close(STDERR_FILENO);

    // Call child program
    try {
        childProgram(&pipeToParent);
    } catch (const std::exception &e) {
        pipeToParent.notifyError(e.what());
        exit(EXIT_FAILURE);
    }

    // Exit (child process shouldn't return to code that created it)
    exit(EXIT_SUCCESS);
}

}
}
