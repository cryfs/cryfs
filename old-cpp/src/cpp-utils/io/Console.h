#pragma once
#ifndef MESSMER_CPPUTILS_IO_CONSOLE_H
#define MESSMER_CPPUTILS_IO_CONSOLE_H

#include <string>
#include <vector>
#include <iostream>
#include <boost/optional.hpp>
#include "../macros.h"
#include "../pointer/unique_ref.h"

namespace cpputils {

class Console {
public:
    virtual ~Console() = default;
    virtual unsigned int ask(const std::string &question, const std::vector<std::string> &options) = 0;
    virtual bool askYesNo(const std::string &question, bool defaultValue) = 0; // NoninteractiveConsole will just return the default value without asking the user.
    virtual void print(const std::string &output) = 0;
    virtual std::string askPassword(const std::string &question) = 0;
};

}


#endif
