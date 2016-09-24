#pragma once
#ifndef MESSMER_CPPUTILS_IO_NONINTERACTIVECONSOLE_H
#define MESSMER_CPPUTILS_IO_NONINTERACTIVECONSOLE_H

#include "Console.h"

namespace cpputils {

    //TODO Add test cases for NoninteractiveConsole
    class NoninteractiveConsole final: public Console {
    public:
        NoninteractiveConsole(unique_ref<Console> baseConsole);

        unsigned int ask(const std::string &question, const std::vector<std::string> &options) override;
        bool askYesNo(const std::string &question, bool defaultValue) override;
        void print(const std::string &output) override;

    private:
        unique_ref<Console> _baseConsole;

        DISALLOW_COPY_AND_ASSIGN(NoninteractiveConsole);
    };

}
#endif
