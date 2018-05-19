#include "subprocess.h"
#include <cstdio>
#include <stdexcept>

#if !defined(_MSC_VER)
#include <sys/wait.h>
constexpr const char* openmode = "re";
#else
#define popen _popen
#define pclose _pclose
#define WEXITSTATUS(a) a
constexpr const char* openmode = "r";
#endif

using std::string;

namespace cpputils {
    //TODO Exception safety

    string Subprocess::call(const string &command) {
        FILE *subprocessOutput = _call(command);

        string result;
        char buffer[1024];
        while(fgets(buffer, sizeof(buffer), subprocessOutput) != nullptr) {
            result += buffer;
        }

        auto returncode = pclose(subprocessOutput);
        if(WEXITSTATUS(returncode) != 0) {
            throw std::runtime_error("Subprocess \""+command+"\" exited with code "+std::to_string(WEXITSTATUS(returncode)));
        }

        return result;
    }

    int Subprocess::callAndGetReturnCode(const string &command) {
        FILE *subprocess = _call(command);

        auto returncode = pclose(subprocess);
        return WEXITSTATUS(returncode);
    }

    FILE *Subprocess::_call(const string &command) {
        FILE *subprocess = popen(command.c_str(), openmode);
        if (!subprocess)
        {
            throw std::runtime_error("Error starting subprocess "+command);
        }
        return subprocess;
    }
}
