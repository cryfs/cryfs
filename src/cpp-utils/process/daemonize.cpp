#include "daemonize.h"
#include "../logging/logging.h"

using namespace cpputils::logging;

//TODO Test daemonize()

#if !defined(_MSC_VER)

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

namespace cpputils {

void daemonize() {
    const pid_t pid = fork();
    if (pid < 0) {
        exit(EXIT_FAILURE);
    }
    if (pid > 0) {
        //We're the parent process. Exit.
        exit(EXIT_SUCCESS);
    }

    // We're the child process.
    umask(0);

    // Create a new SID for the child process
    const pid_t sid = setsid();
    if (sid < 0) {
        LOG(ERR, "Failed to get SID for daemon process");
        exit(EXIT_FAILURE);
    }

    // Change the current working directory to a directory that's always existin
    if ((chdir("/")) < 0) {
        LOG(ERR, "Failed to change working directory for daemon process");
        exit(EXIT_FAILURE);
    }

    // Close out the standard file descriptors. The process can't use them anyhow.
    close(STDIN_FILENO);
    close(STDOUT_FILENO);
    close(STDERR_FILENO);
};
}

#else

#include <Windows.h>

namespace cpputils {

void daemonize() {
	LOG(INFO, "Process started in the background. You can close this console window now.");
	if (!FreeConsole()) {
		LOG(ERR, "Failed to call FreeConsole()");
		exit(EXIT_FAILURE);
	}
}

}

#endif
