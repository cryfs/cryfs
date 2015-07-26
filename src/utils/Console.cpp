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

unsigned int IOStreamConsole::ask(const string &question, const vector<string> &options) {
    if(options.size() == 0) {
        throw std::invalid_argument("options should have at least one entry");
    }
    _output << question << "\n";
    for (unsigned int i = 0; i < options.size(); ++i) {
        _output << " [" << (i+1) << "] " << options[i] << "\n";
    }
    optional<int> choice;
    do {
        _output << "Your choice [1-" << options.size() << "]: " << flush;
        string choiceStr;
        getline(_input, choiceStr);
        choice = parseInt(choiceStr);
    } while(choice == none || *choice < 1 || static_cast<unsigned int>(*choice) > options.size());
    return *choice-1;
}

void IOStreamConsole::print(const std::string &output) {
  _output << output << std::flush;
}
