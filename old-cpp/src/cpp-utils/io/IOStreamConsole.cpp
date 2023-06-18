#include "IOStreamConsole.h"
#include <boost/algorithm/string/trim.hpp>
#include "DontEchoStdinToStdoutRAII.h"
#include <cpp-utils/assert/assert.h>

using std::ostream;
using std::istream;
using std::string;
using std::vector;
using std::flush;
using std::function;
using boost::optional;
using boost::none;

namespace cpputils {

IOStreamConsole::IOStreamConsole(): IOStreamConsole(std::cout, std::cin) {
}

IOStreamConsole::IOStreamConsole(ostream &output, istream &input): _output(output), _input(input) {
}

optional<int> IOStreamConsole::_parseInt(const string &str) {
    try {
        string trimmed = str;
        boost::algorithm::trim(trimmed);
        int parsed = std::stoi(str);
        if (std::to_string(parsed) != trimmed) {
            return none;
        }
        return parsed;
    } catch (const std::invalid_argument &e) {
        return none;
    } catch (const std::out_of_range &e) {
        return none;
    }
}

function<optional<unsigned int>(const string &input)> IOStreamConsole::_parseUIntWithMinMax(unsigned int min, unsigned int max) {
    return [min, max] (const string &input) {
        optional<int> parsed = _parseInt(input);
        if (parsed == none) {
            return optional<unsigned int>(none);
        }
        unsigned int value = static_cast<unsigned int>(*parsed);
        if (value < min || value > max) {
            return optional<unsigned int>(none);
        }
        return optional<unsigned int>(value);
    };
}

template<typename Return>
Return IOStreamConsole::_askForChoice(const string &question, function<optional<Return> (const string&)> parse) {
    optional<Return> choice = none;
    do {
        _output << question << flush;
        string choiceStr;
        getline(_input, choiceStr);
        choice = parse(choiceStr);
    } while(choice == none);
    return *choice;
}

unsigned int IOStreamConsole::ask(const string &question, const vector<string> &options) {
    if(options.size() == 0) {
        throw std::invalid_argument("options should have at least one entry");
    }
    _output << question << "\n";
    for (size_t i = 0; i < options.size(); ++i) {
        _output << " [" << (i+1) << "] " << options[i] << "\n";
    }
    int choice = _askForChoice("Your choice [1-" + std::to_string(options.size()) + "]: ", _parseUIntWithMinMax(1, options.size()));
    return choice-1;
}

function<optional<bool>(const string &input)> IOStreamConsole::_parseYesNo() {
    return [] (const string &input) {
        string trimmed = input;
        boost::algorithm::trim(trimmed);
        if(trimmed == "Y" || trimmed == "y" || trimmed == "Yes" || trimmed == "yes") {
            return optional<bool>(true);
        } else if (trimmed == "N" || trimmed == "n" || trimmed == "No" || trimmed == "no") {
            return optional<bool>(false);
        } else {
            return optional<bool>(none);
        }
    };
}

bool IOStreamConsole::askYesNo(const string &question, bool /*defaultValue*/) {
    _output << question << "\n";
    return _askForChoice("Your choice [y/n]: ", _parseYesNo());
}

void IOStreamConsole::print(const string &output) {
    _output << output << std::flush;
}

string IOStreamConsole::askPassword(const string &question) {
    DontEchoStdinToStdoutRAII _stdin_input_is_hidden_as_long_as_this_is_in_scope;

    _output << question << std::flush;
    string result;
    std::getline(_input, result);
    _output << std::endl;

    ASSERT(result.size() == 0 || result[result.size() - 1] != '\n', "Unexpected std::getline() behavior");

    return result;
}

}
