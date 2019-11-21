#include "ExtPassConsole.h"

#include <cpp-utils/process/subprocess.h>

using std::string;
using std::vector;
using std::shared_ptr;

namespace cpputils {

ExtPassConsole::ExtPassConsole(const std::string & extpass, shared_ptr<Console> baseConsole)
    : _baseConsole(std::move(baseConsole))
    , _extpass(extpass) {
}

bool ExtPassConsole::askYesNo(const string &question, bool defaultValue) {
    return _baseConsole->askYesNo(question, defaultValue);
}

void ExtPassConsole::print(const std::string &output) {
    _baseConsole->print(output);
}

unsigned int ExtPassConsole::ask(const string &question, const vector<string> &options) {
    return _baseConsole->ask(question, options);
}

string ExtPassConsole::askPassword(const string &question) {
    (void)question;
    auto result = cpputils::Subprocess::call(_extpass);

    string output = result.output;

    while(output.back() == '\n' || output.back() == '\r')
        output.erase(output.back());

    return output;
}

}
