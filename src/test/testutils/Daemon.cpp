#include "Daemon.h"

#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <signal.h>

using std::function;

Daemon::Daemon(function<void()> runnable)
  : _runnable(runnable), _child_pid(0) {
}

void Daemon::start() {
  _child_pid = fork();
  if (_child_pid == 0) {
    _runnable();
    exit(0);
  }
}

void Daemon::stop() {
  int retval = kill(_child_pid, SIGINT);
  if (retval != 0) {
    throw std::runtime_error("Failed killing child process");
  }
  int status;
  pid_t pid = waitpid(_child_pid, &status, 0);
  if (pid != _child_pid) {
    throw std::runtime_error("Failed waiting for child process to die");
  }
}
