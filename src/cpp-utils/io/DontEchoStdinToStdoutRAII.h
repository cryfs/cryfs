#pragma once
#ifndef MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H
#define MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H

#include <memory>
#include "../macros.h"

/**
 * If you create an instance of this class in your scope, then any user input from stdin
 * won't be echoed back to stdout until the instance leaves the scope.
 * This can be very handy for password inputs where you don't want the password to be visible on screen.
 */

#if !defined(_MSC_VER)

#include <termios.h>
#include <unistd.h>

namespace cpputils {

class DontEchoStdinToStdoutRAII final {
public:
	DontEchoStdinToStdoutRAII() : _old_state() {
		tcgetattr(STDIN_FILENO, &_old_state);
		termios new_state = _old_state;
		new_state.c_lflag &= ~ECHO;
		tcsetattr(STDIN_FILENO, TCSANOW, &new_state);
	}

	~DontEchoStdinToStdoutRAII() {
		tcsetattr(STDIN_FILENO, TCSANOW, &_old_state);
	}

private:
	termios _old_state;

	DISALLOW_COPY_AND_ASSIGN(DontEchoStdinToStdoutRAII);
};

}

#else

#include <windows.h>

namespace cpputils {

class DontEchoStdinToStdoutRAII final {
public:
	DontEchoStdinToStdoutRAII() : _old_state() {
		HANDLE hStdin = GetStdHandle(STD_INPUT_HANDLE);
		GetConsoleMode(hStdin, &_old_state);
		SetConsoleMode(hStdin, _old_state & (~ENABLE_ECHO_INPUT));
	}

	~DontEchoStdinToStdoutRAII() {
		HANDLE hStdin = GetStdHandle(STD_INPUT_HANDLE);
		SetConsoleMode(hStdin, _old_state);
	}

private:
	DWORD _old_state;

	DISALLOW_COPY_AND_ASSIGN(DontEchoStdinToStdoutRAII);
};

}

#endif

#endif
