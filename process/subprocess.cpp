#include "subprocess.h"
#include <cstdio>
#include <stdexcept>

using std::string;

namespace cpputils {
    string Subprocess::call(const string &command) {
        //TODO Exception safety
        FILE *subprocessOutput = popen(command.c_str(), "r");
        if (!subprocessOutput)
        {
            throw std::runtime_error("Error starting subprocess "+command);
        }

        string result;
        char buffer[1024];
        while(fgets(buffer, sizeof(buffer), subprocessOutput) != NULL) {
            result += buffer;
        }

        auto returncode = pclose(subprocessOutput);
        if(WEXITSTATUS(returncode) != 0) {
            throw std::runtime_error("Subprocess \""+command+"\" exited with code "+std::to_string(WEXITSTATUS(returncode)));
        }

        return result;
    }
}