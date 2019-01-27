#pragma once
#ifndef MESSMER_CPPUTILS_IO_PROGRESSBAR_H
#define MESSMER_CPPUTILS_IO_PROGRESSBAR_H

#include <cpp-utils/macros.h>
#include <string>
#include <memory>
#include "Console.h"

namespace cpputils {

class ProgressBar final {
public:
    explicit ProgressBar(std::shared_ptr<Console> console, const char* preamble, uint64_t max_value);
    explicit ProgressBar(const char* preamble, uint64_t max_value);

    void update(uint64_t value);

private:

    std::shared_ptr<Console> _console;
    std::string _preamble;
    uint64_t _max_value;
    size_t _lastPercentage;

    DISALLOW_COPY_AND_ASSIGN(ProgressBar);
};

}

#endif
