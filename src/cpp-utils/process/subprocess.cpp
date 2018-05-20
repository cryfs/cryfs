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

    namespace {
    FILE *_call(const string &command) {
        FILE *subprocess = popen(command.c_str(), openmode);
        if (!subprocess)
        {
            throw std::runtime_error("Error starting subprocess "+command);
        }
        return subprocess;
    }

    string _getOutput(FILE *subprocess) {
        string output;
        char buffer[1024];
        while(fgets(buffer, sizeof(buffer), subprocess) != nullptr) {
            output += buffer;
        }
        return output;
    }

    int _close(FILE *subprocess) {
        auto returncode = pclose(subprocess);
        if(returncode == -1) {
            throw std::runtime_error("Error calling pclose. Errno: " + std::to_string(errno));
        }
        if (WIFEXITED(returncode) == 0) {
            // WEXITSTATUS is only valud if WIFEXITED is 0.
            throw std::runtime_error("WIFEXITED returned " + std::to_string(WIFEXITED(returncode)));
        }
        return WEXITSTATUS(returncode);
    }
    }

    SubprocessResult Subprocess::call(const string &command) {
        FILE *subprocess = _call(command);
        string output = _getOutput(subprocess);
        int exitcode = _close(subprocess);

        return SubprocessResult {output, exitcode};
    }

    SubprocessResult Subprocess::check_call(const string &command) {
        auto result = call(command);
        if(result.exitcode != 0) {
            throw SubprocessError("Subprocess \""+command+"\" exited with code "+std::to_string(result.exitcode));
        }
        return result;
    }

}
