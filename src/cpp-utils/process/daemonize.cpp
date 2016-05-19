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
#include "../logging/logging.h"

using namespace cpputils::logging;

using std::function;

namespace cpputils {

    //TODO Test daemonize()

    void daemonize(function<void ()> childProgram) {
            pid_t pid = fork();
            if (pid < 0) {
                throw std::runtime_error("fork() failed.");
            }
            if (pid > 0) {
                // We're the parent process. Return.
                return;
            }

            // We're the child process.
            umask(0);

            // Create a new SID for the child process
            pid_t sid = setsid();
            if (sid < 0) {
                LOG(ERROR) << "Failed to get SID for pipe process";
                exit(EXIT_FAILURE);
            }

            // Change the current working directory to a directory that's always existing
            if ((chdir("/")) < 0) {
                LOG(ERROR) << "Failed to change working directory for pipe process";
                exit(EXIT_FAILURE);
            }

            // Close out the standard file descriptors. The process can't use them anyhow.
            close(STDIN_FILENO);
            close(STDOUT_FILENO);
            close(STDERR_FILENO);

            // Call child program
            childProgram();

            // Exit (child process shouldn't return to code that created it)
            exit(EXIT_SUCCESS);
    };
}
