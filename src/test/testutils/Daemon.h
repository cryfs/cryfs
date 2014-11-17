#pragma once
#ifndef TEST_TESTUTILS_DAEMON_H_
#define TEST_TESTUTILS_DAEMON_H_

#include <functional>

class Daemon {
public:
  Daemon(std::function<void()> runnable);
  void start();
  void stop();

private:
  std::function<void()> _runnable;
  pid_t _child_pid;
};

#endif
