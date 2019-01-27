#include "ProgressBar.h"
#include <iostream>
#include <limits>
#include <mutex>
#include "IOStreamConsole.h"

using std::string;

namespace cpputils {

ProgressBar::ProgressBar(const char* preamble, uint64_t max_value)
: ProgressBar(std::make_shared<IOStreamConsole>(), preamble, max_value) {}

ProgressBar::ProgressBar(std::shared_ptr<Console> console, const char* preamble, uint64_t max_value)
: _console(std::move(console))
, _preamble(string("\r") + preamble + " ")
, _max_value(max_value)
, _lastPercentage(std::numeric_limits<decltype(_lastPercentage)>::max()) {
    ASSERT(_max_value > 0, "Progress bar can't handle max_value of 0");

    _console->print("\n");

    // show progress bar. _lastPercentage is different to zero, so it shows.
    update(0);
}

void ProgressBar::update(uint64_t value) {
    const size_t percentage = (100 * value) / _max_value;
    if (percentage != _lastPercentage) {
        _console->print(_preamble + std::to_string(percentage) + "%");
        _lastPercentage = percentage;
    }
}

}
