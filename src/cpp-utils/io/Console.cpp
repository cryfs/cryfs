#include "Console.h"

#include <boost/optional.hpp>
#include <boost/algorithm/string/trim.hpp>

using std::string;
using std::vector;
using std::ostream;
using std::istream;
using std::flush;
using std::getline;
using boost::optional;
using boost::none;
using std::function;

using namespace cpputils;

IOStreamConsole::IOStreamConsole(): IOStreamConsole(std::cout, std::cin) {
}

IOStreamConsole::IOStreamConsole(ostream &output, istream &input): _output(output), _input(input) {
}

optional<int> parseInt(const string &str) {
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

function<optional<unsigned int>(const std::string &input)> parseUIntWithMinMax(unsigned int min, unsigned int max) {
    return [min, max] (const string &input) {
        optional<int> parsed = parseInt(input);
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
    for (unsigned int i = 0; i < options.size(); ++i) {
        _output << " [" << (i+1) << "] " << options[i] << "\n";
    }
    int choice = _askForChoice("Your choice [1-" + std::to_string(options.size()) + "]: ", parseUIntWithMinMax(1, options.size()));
    return choice-1;
}

function<optional<bool>(const string &input)> parseYesNo() {
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

bool IOStreamConsole::askYesNo(const string &question) {
    _output << question << "\n";
    return _askForChoice("Your choice [y/n]: ", parseYesNo());
}

void IOStreamConsole::print(const std::string &output) {
  _output << output << std::flush;
}
