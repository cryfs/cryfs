#pragma once
#ifndef MESSMER_CPPUTILS_ASSERT_BACKTRACE_H
#define MESSMER_CPPUTILS_ASSERT_BACKTRACE_H

#include <string>

namespace cpputils {
    std::string backtrace();

    //TODO Refactor (for example: RAII or at least try{}finally{} instead of  free())
    //TODO Use the following? https://github.com/bombela/backward-cpp
    void showBacktraceOnCrash();

    // Like showBacktraceOnCrash(), but only handles the signals that mean the process actually
    // crashed (SIGSEGV, SIGBUS, SIGILL, SIGFPE) and doesn't terminate the process itself.
    // After printing the backtrace, it resets the signal to its default handler and re-raises it,
    // so the process still dies from that signal and still writes a core dump.
    // Use this instead of showBacktraceOnCrash() in programs that use SIGABRT for something else,
    // e.g. our test binaries, where ASSERT() and gtest death tests deliberately abort().
    // Once this is installed, showBacktraceOnCrash() is a no-op, so that code called by such a
    // program (e.g. cryfs_cli::Cli::main(), which the CLI tests run in-process) can't displace
    // these handlers with ones that swallow the signal.
    void showBacktraceOnCrashSignals();
}

#endif
