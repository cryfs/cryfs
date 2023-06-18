#include "DontEchoStdinToStdoutRAII.h"

#if !defined(_MSC_VER)

#include <termios.h>
#include <unistd.h>

namespace cpputils
{
    namespace details
    {

        class DontEchoStdinToStdoutRAII_ final
        {
        public:
            DontEchoStdinToStdoutRAII_() : _old_state()
            {
                tcgetattr(STDIN_FILENO, &_old_state);
                termios new_state = _old_state;
                new_state.c_lflag &= ~ECHO;
                tcsetattr(STDIN_FILENO, TCSANOW, &new_state);
            }

            ~DontEchoStdinToStdoutRAII_()
            {
                tcsetattr(STDIN_FILENO, TCSANOW, &_old_state);
            }

        private:
            termios _old_state;

            DISALLOW_COPY_AND_ASSIGN(DontEchoStdinToStdoutRAII_);
        };

    }
}

#else

#include <windows.h>

namespace cpputils
{
    namespace details
    {

        class DontEchoStdinToStdoutRAII_ final
        {
        public:
            DontEchoStdinToStdoutRAII_() : _old_state()
            {
                HANDLE hStdin = GetStdHandle(STD_INPUT_HANDLE);
                GetConsoleMode(hStdin, &_old_state);
                SetConsoleMode(hStdin, _old_state & (~ENABLE_ECHO_INPUT));
            }

            ~DontEchoStdinToStdoutRAII_()
            {
                HANDLE hStdin = GetStdHandle(STD_INPUT_HANDLE);
                SetConsoleMode(hStdin, _old_state);
            }

        private:
            DWORD _old_state;

            DISALLOW_COPY_AND_ASSIGN(DontEchoStdinToStdoutRAII_);
        };

    }
}

#endif

using cpputils::make_unique_ref;

namespace cpputils
{

    DontEchoStdinToStdoutRAII::DontEchoStdinToStdoutRAII()
        : raii(make_unique_ref<details::DontEchoStdinToStdoutRAII_>()) {}

    DontEchoStdinToStdoutRAII::~DontEchoStdinToStdoutRAII() {}

}
