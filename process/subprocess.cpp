#include "subprocess.h"
#include <cstdio>
#include <stdexcept>
#include <sys/wait.h>

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
        FILE *subprocess = popen(command.c_str(), "r");
        if (!subprocess)
        {
            throw std::runtime_error("Error starting subprocess "+command);
        }
        return subprocess;
    }
}
