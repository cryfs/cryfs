#pragma once
#ifndef MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H
#define MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H

#include <iostream>
#include <string>
#include <termios.h>
#include <unistd.h>
#include "../macros.h"

namespace cpputils {

/**
 * If you create an instance of this class in your scope, then any user input from stdin
 * won't be echoed back to stdout until the instance leaves the scope.
 * This can be very handy for password inputs where you don't want the password to be visible on screen.
 */
class DontEchoStdinToStdoutRAII final {
public:
    DontEchoStdinToStdoutRAII() {
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

#endif
