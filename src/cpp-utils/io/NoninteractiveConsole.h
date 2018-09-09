#pragma once
#ifndef MESSMER_CPPUTILS_IO_NONINTERACTIVECONSOLE_H
#define MESSMER_CPPUTILS_IO_NONINTERACTIVECONSOLE_H

#include "Console.h"

namespace cpputils {

    //TODO Add test cases for NoninteractiveConsole
    class NoninteractiveConsole final: public Console {
    public:
        NoninteractiveConsole(std::shared_ptr<Console> baseConsole);

        unsigned int ask(const std::string &question, const std::vector<std::string> &options) override;
        bool askYesNo(const std::string &question, bool defaultValue) override;
        void print(const std::string &output) override;
        std::string askPassword(const std::string &question) override;

    private:
        std::shared_ptr<Console> _baseConsole;

        DISALLOW_COPY_AND_ASSIGN(NoninteractiveConsole);
    };

}
#endif
